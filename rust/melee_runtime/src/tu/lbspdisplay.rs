//! Port of src/melee/lb/lbspdisplay.c (light list loader, render-flag walkers, JObj index finders, EFB blur camera). Callees via `ext`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_core::fp::fmadds;

// // BEGIN GENERATED EXT (rust/melee_runtime/gen/lbtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code, unreachable_code)]
    use super::*;
    use core::ffi::c_void;
    pub mod hook {
        use super::*;
        pub static mut GObj_Create: Option<fn(u16, u8, u8) -> *mut HSD_GObj> = None;
        pub static mut GObj_InitUserData: Option<fn(*mut HSD_GObj, u8, Option<unsafe extern "C" fn(*mut c_void)>, *mut c_void)> = None;
        pub static mut GObj_SetupGXLinkMax: Option<fn(*mut HSD_GObj, Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, u32)> = None;
        pub static mut GXBegin: Option<fn(u32, u32, u16)> = None;
        pub static mut GXClearVtxDesc: Option<fn()> = None;
        pub static mut GXGetTexBufferSize: Option<fn(u16, u16, u32, u8, u8) -> u32> = None;
        pub static mut GXGetTexObjFmt: Option<fn(*const _GXTexObj) -> u32> = None;
        pub static mut GXGetTexObjHeight: Option<fn(*const _GXTexObj) -> u16> = None;
        pub static mut GXGetTexObjWidth: Option<fn(*const _GXTexObj) -> u16> = None;
        pub static mut GXInitTexObj: Option<fn(*mut _GXTexObj, *mut c_void, u16, u16, u32, u32, u32, u8)> = None;
        pub static mut GXLoadPosMtxImm: Option<fn(*mut [f32; 4], u32)> = None;
        pub static mut GXLoadTexObj: Option<fn(*mut _GXTexObj, u32)> = None;
        pub static mut GXPosition2f32: Option<fn(f32, f32)> = None;
        pub static mut GXSetAlphaCompare: Option<fn(u32, u8, u32, u32, u8)> = None;
        pub static mut GXSetBlendMode: Option<fn(u32, u32, u32, u32)> = None;
        pub static mut GXSetColorUpdate: Option<fn(u8)> = None;
        pub static mut GXSetCullMode: Option<fn(u32)> = None;
        pub static mut GXSetCurrentMtx: Option<fn(u32)> = None;
        pub static mut GXSetNumTevStages: Option<fn(u8)> = None;
        pub static mut GXSetNumTexGens: Option<fn(u8)> = None;
        pub static mut GXSetScissor: Option<fn(u32, u32, u32, u32)> = None;
        pub static mut GXSetTevAlphaIn: Option<fn(u32, u32, u32, u32, u32)> = None;
        pub static mut GXSetTevAlphaOp: Option<fn(u32, u32, u32, u32, u8, u32)> = None;
        pub static mut GXSetTevColor: Option<fn(u32, _GXColor)> = None;
        pub static mut GXSetTevColorIn: Option<fn(u32, u32, u32, u32, u32)> = None;
        pub static mut GXSetTevColorOp: Option<fn(u32, u32, u32, u32, u8, u32)> = None;
        pub static mut GXSetTevKColor: Option<fn(u32, _GXColor)> = None;
        pub static mut GXSetTevKColorSel: Option<fn(u32, u32)> = None;
        pub static mut GXSetTevOrder: Option<fn(u32, u32, u32, u32)> = None;
        pub static mut GXSetTevSwapMode: Option<fn(u32, u32, u32)> = None;
        pub static mut GXSetTexCoordGen2: Option<fn(u32, u32, u32, u32, u8, u32)> = None;
        pub static mut GXSetVtxAttrFmt: Option<fn(u32, u32, u32, u32, u8)> = None;
        pub static mut GXSetVtxDesc: Option<fn(u32, u32)> = None;
        pub static mut GXSetZMode: Option<fn(u8, u32, u8)> = None;
        pub static mut GXTexCoord2f32: Option<fn(f32, f32)> = None;
        pub static mut HSD_CObjAlloc: Option<fn() -> *mut HSD_CObj> = None;
        pub static mut HSD_CObjGetAuthoredAspect: Option<fn(*mut HSD_CObj) -> f32> = None;
        pub static mut HSD_CObjGetCurrent: Option<fn() -> *mut HSD_CObj> = None;
        pub static mut HSD_CObjGetProjectionType: Option<fn(*mut HSD_CObj) -> i32> = None;
        pub static mut HSD_CObjGetScissor: Option<fn(*mut HSD_CObj, *mut _Scissor)> = None;
        pub static mut HSD_CObjGetViewingMtx: Option<fn(*mut HSD_CObj, *mut [f32; 4])> = None;
        pub static mut HSD_CObjLoadDesc: Option<fn(*mut HSD_CObjDesc) -> *mut HSD_CObj> = None;
        pub static mut HSD_CObjSetAspect: Option<fn(*mut HSD_CObj, f32)> = None;
        pub static mut HSD_CObjSetCurrent: Option<fn(*mut HSD_CObj) -> i32> = None;
        pub static mut HSD_CObjSetEyePosition: Option<fn(*mut HSD_CObj, *mut Vec)> = None;
        pub static mut HSD_CObjSetFar: Option<fn(*mut HSD_CObj, f32)> = None;
        pub static mut HSD_CObjSetInterest: Option<fn(*mut HSD_CObj, *mut Vec)> = None;
        pub static mut HSD_CObjSetNear: Option<fn(*mut HSD_CObj, f32)> = None;
        pub static mut HSD_CObjSetOrtho: Option<fn(*mut HSD_CObj, f32, f32, f32, f32)> = None;
        pub static mut HSD_CObjSetProjectionType: Option<fn(*mut HSD_CObj, u32)> = None;
        pub static mut HSD_CObjSetRoll: Option<fn(*mut HSD_CObj, f32)> = None;
        pub static mut HSD_CObjSetScissor: Option<fn(*mut HSD_CObj, *mut _Scissor)> = None;
        pub static mut HSD_CObjSetViewport: Option<fn(*mut HSD_CObj, *mut _HSD_RectS16)> = None;
        pub static mut HSD_Free: Option<fn(*mut c_void)> = None;
        pub static mut HSD_GObjObject_80390A70: Option<fn(*mut HSD_GObj, u8, *mut c_void)> = None;
        pub static mut HSD_ImageDescCopyFromEFB: Option<fn(*mut HSD_ImageDesc, u16, u16, u8, i32)> = None;
        pub static mut HSD_LObjAddAnimAll: Option<fn(*mut HSD_LObj, *mut HSD_LightAnim)> = None;
        pub static mut HSD_LObjLoadDesc: Option<fn(*mut HSD_LightDesc) -> *mut HSD_LObj> = None;
        pub static mut HSD_MemAlloc: Option<fn(i32) -> *mut c_void> = None;
        pub static mut HSD_SetupChannel: Option<fn(*mut HSD_Chan)> = None;
        pub static mut HSD_StateInvalidate: Option<fn(i32)> = None;
        pub static mut HSD_StateSetNumChans: Option<fn(i32)> = None;
        pub static mut HSD_StateSetZMode: Option<fn(i32, i32, i32)> = None;
        pub static mut __builtin_va_end: Option<fn(c_void /* char *& */)> = None;
        pub static mut __va_arg: Option<fn(*mut i8, u32) -> *mut c_void> = None;
        pub static mut lbDvd_GetPreloadedArchive: Option<fn(i32) -> *mut c_void> = None;
    }
    pub fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj {
        match unsafe { hook::GObj_Create } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: GObj_Create is not ported") } }
    }
    pub fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void) {
        match unsafe { hook::GObj_InitUserData } { Some(f) => f(a0, a1, a2, a3), None => { panic!("TODO: GObj_InitUserData is not ported") } }
    }
    pub fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32) {
        match unsafe { hook::GObj_SetupGXLinkMax } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: GObj_SetupGXLinkMax is not ported") } }
    }
    pub fn GXBegin(a0: u32, a1: u32, a2: u16) {
        match unsafe { hook::GXBegin } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: GXBegin is not ported") } }
    }
    pub fn GXClearVtxDesc() {
        match unsafe { hook::GXClearVtxDesc } { Some(f) => f(), None => { panic!("TODO: GXClearVtxDesc is not ported") } }
    }
    pub fn GXGetTexBufferSize(a0: u16, a1: u16, a2: u32, a3: u8, a4: u8) -> u32 {
        match unsafe { hook::GXGetTexBufferSize } { Some(f) => f(a0, a1, a2, a3, a4), None => { panic!("TODO: GXGetTexBufferSize is not ported") } }
    }
    pub fn GXGetTexObjFmt(a0: *const _GXTexObj) -> u32 {
        match unsafe { hook::GXGetTexObjFmt } { Some(f) => f(a0), None => { panic!("TODO: GXGetTexObjFmt is not ported") } }
    }
    pub fn GXGetTexObjHeight(a0: *const _GXTexObj) -> u16 {
        match unsafe { hook::GXGetTexObjHeight } { Some(f) => f(a0), None => { panic!("TODO: GXGetTexObjHeight is not ported") } }
    }
    pub fn GXGetTexObjWidth(a0: *const _GXTexObj) -> u16 {
        match unsafe { hook::GXGetTexObjWidth } { Some(f) => f(a0), None => { panic!("TODO: GXGetTexObjWidth is not ported") } }
    }
    pub fn GXInitTexObj(a0: *mut _GXTexObj, a1: *mut c_void, a2: u16, a3: u16, a4: u32, a5: u32, a6: u32, a7: u8) {
        match unsafe { hook::GXInitTexObj } { Some(f) => f(a0, a1, a2, a3, a4, a5, a6, a7), None => { panic!("TODO: GXInitTexObj is not ported") } }
    }
    pub fn GXLoadPosMtxImm(a0: *mut [f32; 4], a1: u32) {
        match unsafe { hook::GXLoadPosMtxImm } { Some(f) => f(a0, a1), None => { panic!("TODO: GXLoadPosMtxImm is not ported") } }
    }
    pub fn GXLoadTexObj(a0: *mut _GXTexObj, a1: u32) {
        match unsafe { hook::GXLoadTexObj } { Some(f) => f(a0, a1), None => { panic!("TODO: GXLoadTexObj is not ported") } }
    }
    pub fn GXPosition2f32(a0: f32, a1: f32) {
        match unsafe { hook::GXPosition2f32 } { Some(f) => f(a0, a1), None => { panic!("TODO: GXPosition2f32 is not ported") } }
    }
    pub fn GXSetAlphaCompare(a0: u32, a1: u8, a2: u32, a3: u32, a4: u8) {
        match unsafe { hook::GXSetAlphaCompare } { Some(f) => f(a0, a1, a2, a3, a4), None => { panic!("TODO: GXSetAlphaCompare is not ported") } }
    }
    pub fn GXSetBlendMode(a0: u32, a1: u32, a2: u32, a3: u32) {
        match unsafe { hook::GXSetBlendMode } { Some(f) => f(a0, a1, a2, a3), None => { panic!("TODO: GXSetBlendMode is not ported") } }
    }
    pub fn GXSetColorUpdate(a0: u8) {
        match unsafe { hook::GXSetColorUpdate } { Some(f) => f(a0), None => { panic!("TODO: GXSetColorUpdate is not ported") } }
    }
    pub fn GXSetCullMode(a0: u32) {
        match unsafe { hook::GXSetCullMode } { Some(f) => f(a0), None => { panic!("TODO: GXSetCullMode is not ported") } }
    }
    pub fn GXSetCurrentMtx(a0: u32) {
        match unsafe { hook::GXSetCurrentMtx } { Some(f) => f(a0), None => { panic!("TODO: GXSetCurrentMtx is not ported") } }
    }
    pub fn GXSetNumTevStages(a0: u8) {
        match unsafe { hook::GXSetNumTevStages } { Some(f) => f(a0), None => { panic!("TODO: GXSetNumTevStages is not ported") } }
    }
    pub fn GXSetNumTexGens(a0: u8) {
        match unsafe { hook::GXSetNumTexGens } { Some(f) => f(a0), None => { panic!("TODO: GXSetNumTexGens is not ported") } }
    }
    pub fn GXSetScissor(a0: u32, a1: u32, a2: u32, a3: u32) {
        match unsafe { hook::GXSetScissor } { Some(f) => f(a0, a1, a2, a3), None => { panic!("TODO: GXSetScissor is not ported") } }
    }
    pub fn GXSetTevAlphaIn(a0: u32, a1: u32, a2: u32, a3: u32, a4: u32) {
        match unsafe { hook::GXSetTevAlphaIn } { Some(f) => f(a0, a1, a2, a3, a4), None => { panic!("TODO: GXSetTevAlphaIn is not ported") } }
    }
    pub fn GXSetTevAlphaOp(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32) {
        match unsafe { hook::GXSetTevAlphaOp } { Some(f) => f(a0, a1, a2, a3, a4, a5), None => { panic!("TODO: GXSetTevAlphaOp is not ported") } }
    }
    pub fn GXSetTevColor(a0: u32, a1: _GXColor) {
        match unsafe { hook::GXSetTevColor } { Some(f) => f(a0, a1), None => { panic!("TODO: GXSetTevColor is not ported") } }
    }
    pub fn GXSetTevColorIn(a0: u32, a1: u32, a2: u32, a3: u32, a4: u32) {
        match unsafe { hook::GXSetTevColorIn } { Some(f) => f(a0, a1, a2, a3, a4), None => { panic!("TODO: GXSetTevColorIn is not ported") } }
    }
    pub fn GXSetTevColorOp(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32) {
        match unsafe { hook::GXSetTevColorOp } { Some(f) => f(a0, a1, a2, a3, a4, a5), None => { panic!("TODO: GXSetTevColorOp is not ported") } }
    }
    pub fn GXSetTevKColor(a0: u32, a1: _GXColor) {
        match unsafe { hook::GXSetTevKColor } { Some(f) => f(a0, a1), None => { panic!("TODO: GXSetTevKColor is not ported") } }
    }
    pub fn GXSetTevKColorSel(a0: u32, a1: u32) {
        match unsafe { hook::GXSetTevKColorSel } { Some(f) => f(a0, a1), None => { panic!("TODO: GXSetTevKColorSel is not ported") } }
    }
    pub fn GXSetTevOrder(a0: u32, a1: u32, a2: u32, a3: u32) {
        match unsafe { hook::GXSetTevOrder } { Some(f) => f(a0, a1, a2, a3), None => { panic!("TODO: GXSetTevOrder is not ported") } }
    }
    pub fn GXSetTevSwapMode(a0: u32, a1: u32, a2: u32) {
        match unsafe { hook::GXSetTevSwapMode } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: GXSetTevSwapMode is not ported") } }
    }
    pub fn GXSetTexCoordGen2(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32) {
        match unsafe { hook::GXSetTexCoordGen2 } { Some(f) => f(a0, a1, a2, a3, a4, a5), None => { panic!("TODO: GXSetTexCoordGen2 is not ported") } }
    }
    pub fn GXSetVtxAttrFmt(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8) {
        match unsafe { hook::GXSetVtxAttrFmt } { Some(f) => f(a0, a1, a2, a3, a4), None => { panic!("TODO: GXSetVtxAttrFmt is not ported") } }
    }
    pub fn GXSetVtxDesc(a0: u32, a1: u32) {
        match unsafe { hook::GXSetVtxDesc } { Some(f) => f(a0, a1), None => { panic!("TODO: GXSetVtxDesc is not ported") } }
    }
    pub fn GXSetZMode(a0: u8, a1: u32, a2: u8) {
        match unsafe { hook::GXSetZMode } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: GXSetZMode is not ported") } }
    }
    pub fn GXTexCoord2f32(a0: f32, a1: f32) {
        match unsafe { hook::GXTexCoord2f32 } { Some(f) => f(a0, a1), None => { panic!("TODO: GXTexCoord2f32 is not ported") } }
    }
    pub fn HSD_CObjAlloc() -> *mut HSD_CObj {
        match unsafe { hook::HSD_CObjAlloc } { Some(f) => f(), None => { panic!("TODO: HSD_CObjAlloc is not ported") } }
    }
    pub fn HSD_CObjGetAuthoredAspect(a0: *mut HSD_CObj) -> f32 {
        match unsafe { hook::HSD_CObjGetAuthoredAspect } { Some(f) => f(a0), None => { panic!("TODO: HSD_CObjGetAuthoredAspect is not ported") } }
    }
    pub fn HSD_CObjGetCurrent() -> *mut HSD_CObj {
        match unsafe { hook::HSD_CObjGetCurrent } { Some(f) => f(), None => { panic!("TODO: HSD_CObjGetCurrent is not ported") } }
    }
    pub fn HSD_CObjGetProjectionType(a0: *mut HSD_CObj) -> i32 {
        match unsafe { hook::HSD_CObjGetProjectionType } { Some(f) => f(a0), None => { panic!("TODO: HSD_CObjGetProjectionType is not ported") } }
    }
    pub fn HSD_CObjGetScissor(a0: *mut HSD_CObj, a1: *mut _Scissor) {
        match unsafe { hook::HSD_CObjGetScissor } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjGetScissor is not ported") } }
    }
    pub fn HSD_CObjGetViewingMtx(a0: *mut HSD_CObj, a1: *mut [f32; 4]) {
        match unsafe { hook::HSD_CObjGetViewingMtx } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjGetViewingMtx is not ported") } }
    }
    pub fn HSD_CObjLoadDesc(a0: *mut HSD_CObjDesc) -> *mut HSD_CObj {
        match unsafe { hook::HSD_CObjLoadDesc } { Some(f) => f(a0), None => { panic!("TODO: HSD_CObjLoadDesc is not ported") } }
    }
    pub fn HSD_CObjSetAspect(a0: *mut HSD_CObj, a1: f32) {
        match unsafe { hook::HSD_CObjSetAspect } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjSetAspect is not ported") } }
    }
    pub fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 {
        match unsafe { hook::HSD_CObjSetCurrent } { Some(f) => f(a0), None => { panic!("TODO: HSD_CObjSetCurrent is not ported") } }
    }
    pub fn HSD_CObjSetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec) {
        match unsafe { hook::HSD_CObjSetEyePosition } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjSetEyePosition is not ported") } }
    }
    pub fn HSD_CObjSetFar(a0: *mut HSD_CObj, a1: f32) {
        match unsafe { hook::HSD_CObjSetFar } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjSetFar is not ported") } }
    }
    pub fn HSD_CObjSetInterest(a0: *mut HSD_CObj, a1: *mut Vec) {
        match unsafe { hook::HSD_CObjSetInterest } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjSetInterest is not ported") } }
    }
    pub fn HSD_CObjSetNear(a0: *mut HSD_CObj, a1: f32) {
        match unsafe { hook::HSD_CObjSetNear } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjSetNear is not ported") } }
    }
    pub fn HSD_CObjSetOrtho(a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32) {
        match unsafe { hook::HSD_CObjSetOrtho } { Some(f) => f(a0, a1, a2, a3, a4), None => { panic!("TODO: HSD_CObjSetOrtho is not ported") } }
    }
    pub fn HSD_CObjSetProjectionType(a0: *mut HSD_CObj, a1: u32) {
        match unsafe { hook::HSD_CObjSetProjectionType } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjSetProjectionType is not ported") } }
    }
    pub fn HSD_CObjSetRoll(a0: *mut HSD_CObj, a1: f32) {
        match unsafe { hook::HSD_CObjSetRoll } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjSetRoll is not ported") } }
    }
    pub fn HSD_CObjSetScissor(a0: *mut HSD_CObj, a1: *mut _Scissor) {
        match unsafe { hook::HSD_CObjSetScissor } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjSetScissor is not ported") } }
    }
    pub fn HSD_CObjSetViewport(a0: *mut HSD_CObj, a1: *mut _HSD_RectS16) {
        match unsafe { hook::HSD_CObjSetViewport } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjSetViewport is not ported") } }
    }
    pub fn HSD_Free(a0: *mut c_void) {
        match unsafe { hook::HSD_Free } { Some(f) => f(a0), None => { panic!("TODO: HSD_Free is not ported") } }
    }
    pub fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) {
        match unsafe { hook::HSD_GObjObject_80390A70 } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: HSD_GObjObject_80390A70 is not ported") } }
    }
    pub fn HSD_ImageDescCopyFromEFB(a0: *mut HSD_ImageDesc, a1: u16, a2: u16, a3: u8, a4: i32) {
        match unsafe { hook::HSD_ImageDescCopyFromEFB } { Some(f) => f(a0, a1, a2, a3, a4), None => { panic!("TODO: HSD_ImageDescCopyFromEFB is not ported") } }
    }
    pub fn HSD_LObjAddAnimAll(a0: *mut HSD_LObj, a1: *mut HSD_LightAnim) {
        match unsafe { hook::HSD_LObjAddAnimAll } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_LObjAddAnimAll is not ported") } }
    }
    pub fn HSD_LObjLoadDesc(a0: *mut HSD_LightDesc) -> *mut HSD_LObj {
        match unsafe { hook::HSD_LObjLoadDesc } { Some(f) => f(a0), None => { panic!("TODO: HSD_LObjLoadDesc is not ported") } }
    }
    pub fn HSD_MemAlloc(a0: i32) -> *mut c_void {
        match unsafe { hook::HSD_MemAlloc } { Some(f) => f(a0), None => { panic!("TODO: HSD_MemAlloc is not ported") } }
    }
    pub fn HSD_SetupChannel(a0: *mut HSD_Chan) {
        match unsafe { hook::HSD_SetupChannel } { Some(f) => f(a0), None => { panic!("TODO: HSD_SetupChannel is not ported") } }
    }
    pub fn HSD_StateInvalidate(a0: i32) {
        match unsafe { hook::HSD_StateInvalidate } { Some(f) => f(a0), None => { panic!("TODO: HSD_StateInvalidate is not ported") } }
    }
    pub fn HSD_StateSetNumChans(a0: i32) {
        match unsafe { hook::HSD_StateSetNumChans } { Some(f) => f(a0), None => { panic!("TODO: HSD_StateSetNumChans is not ported") } }
    }
    pub fn HSD_StateSetZMode(a0: i32, a1: i32, a2: i32) {
        match unsafe { hook::HSD_StateSetZMode } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: HSD_StateSetZMode is not ported") } }
    }
    pub fn __builtin_va_end(a0: c_void /* char *& */) {
        match unsafe { hook::__builtin_va_end } { Some(f) => f(a0), None => { panic!("TODO: __builtin_va_end is not ported") } }
    }
    pub fn __va_arg(a0: *mut i8, a1: u32) -> *mut c_void {
        match unsafe { hook::__va_arg } { Some(f) => f(a0, a1), None => { panic!("TODO: __va_arg is not ported") } }
    }
    pub fn lbDvd_GetPreloadedArchive(a0: i32) -> *mut c_void {
        match unsafe { hook::lbDvd_GetPreloadedArchive } { Some(f) => f(a0), None => { panic!("TODO: lbDvd_GetPreloadedArchive is not ported") } }
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObj_CameraKind: u8 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

use melee_types::inline as ji;

#[inline]
fn hsd_assert(cond: bool, what: &str) {
    if !cond { panic!("HSD_ASSERT {what}"); }
}

pub unsafe fn lb_80011AC4(mut list: *mut *mut LightList) -> *mut HSD_LObj {
    let mut prev: *mut HSD_LObj = null_mut();
    let mut first: *mut HSD_LObj = null_mut();
    while !(*list).is_null() {
        let curr = ext::HSD_LObjLoadDesc((**list).desc);
        let anims = (**list).anims;
        if !anims.is_null() {
            ext::HSD_LObjAddAnimAll(curr, *anims);
        }
        if !prev.is_null() {
            hsd_assert(!prev.is_null(), "lobj.h:0x136");
            (*prev).next = curr;
        } else {
            first = curr;
        }
        prev = curr;
        list = list.add(1);
    }
    first
}

pub unsafe fn lb_80011B74(dobj: *mut HSD_DObj, flags: u32) {
    if !(*dobj).next.is_null() {
        lb_80011B74((*dobj).next, flags);
    }
    (*(*dobj).mobj).rendermode |= flags;
}

/// Duplicated in lb_00F9.c.
unsafe fn checkJObjFlags(jobj: *mut HSD_JObj) -> bool {
    (*jobj).flags & ((1 << 5) | (1 << 14)) == 0
}

pub unsafe fn lb_80011C18(jobj: *mut HSD_JObj, flags: u32) {
    let mut cur = (*jobj).child;
    if !cur.is_null() {
        if !(*cur).child.is_null() { lb_80011C18((*cur).child, flags); }
        if !(*cur).next.is_null() { lb_80011C18((*cur).next, flags); }
        if checkJObjFlags(cur) {
            let d = (*cur).u.dobj;
            if !d.is_null() { lb_80011B74(d, flags); }
        }
    }

    cur = (*jobj).next;
    if !cur.is_null() {
        if !(*cur).child.is_null() { lb_80011C18((*cur).child, flags); }
        if !(*cur).next.is_null() { lb_80011C18((*cur).next, flags); }
        if checkJObjFlags(cur) {
            let d = (*cur).u.dobj;
            if !d.is_null() { lb_80011B74(d, flags); }
        }
    }

    if checkJObjFlags(jobj) {
        let mut dobj = (*jobj).u.dobj;
        if !dobj.is_null() {
            let next = (*dobj).next;
            dobj = (*jobj).u.dobj;
            if !next.is_null() { lb_80011B74(next, flags); }
            (*(*dobj).mobj).rendermode |= flags;
        }
    }
}

/// Walk the JObj tree (pre-order, instance joints not descended) to index `target`, resuming
/// from `(jobj, cur)` when the target is not before `prev`.
unsafe fn jobj_walk(root: *mut HSD_JObj, prev: i32, target: i32) -> (*mut HSD_JObj, i32) {
    let mut jobj: *mut HSD_JObj;
    let mut cur: i32;
    jobj = root;
    cur = 0;
    let _ = prev;
    while !jobj.is_null() {
        if cur == target { break; }
        let mut saved = jobj;
        let next_node: *mut HSD_JObj;
        if (*jobj).flags & (JOBJ_INSTANCE as u32) == 0 && !(*jobj).child.is_null() {
            next_node = (*jobj).child;
        } else if !(*jobj).next.is_null() {
            next_node = (*jobj).next;
        } else {
            loop {
                if (*saved).parent.is_null() { next_node = null_mut(); break; }
                if !(*(*saved).parent).next.is_null() {
                    saved = (*(*saved).parent).next;
                    next_node = saved;
                    break;
                }
                saved = (*saved).parent;
            }
        }
        jobj = next_node;
        cur += 1;
    }
    (jobj, cur)
}

/// Shared body of lb_80011E24 / lb_8001204C. The C keeps `jobj`/`cur` across iterations when the
/// next target is not before the previous one, so the walk resumes where it stopped.
unsafe fn jobj_find_many(root: *mut HSD_JObj, result: *mut *mut HSD_JObj, targets: &mut dyn Iterator<Item = i32>) -> i32 {
    let mut found = 0;
    let mut prev: i32 = -1;
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut cur: i32 = 0;
    let mut out = result;
    for target in targets {
        if prev > target || prev == -1 {
            jobj = root;
            cur = 0;
        } else {
            cur = prev;
        }
        while !jobj.is_null() {
            if cur == target { break; }
            let mut saved = jobj;
            let next_node: *mut HSD_JObj;
            if (*jobj).flags & (JOBJ_INSTANCE as u32) == 0 && !(*jobj).child.is_null() {
                next_node = (*jobj).child;
            } else if !(*jobj).next.is_null() {
                next_node = (*jobj).next;
            } else {
                loop {
                    if (*saved).parent.is_null() { next_node = null_mut(); break; }
                    if !(*(*saved).parent).next.is_null() {
                        saved = (*(*saved).parent).next;
                        next_node = saved;
                        break;
                    }
                    saved = (*saved).parent;
                }
            }
            jobj = next_node;
            cur += 1;
        }
        *out = jobj;
        prev = cur;
        out = out.add(1);
        if !jobj.is_null() { found += 1; }
    }
    found
}

/// C: `int lb_80011E24(HSD_JObj* root, HSD_JObj** result, ...)`; the variadic list is `indices`
/// (read up to the first -1 terminator, which need not be present).
pub unsafe fn lb_80011E24(root: *mut HSD_JObj, result: *mut *mut HSD_JObj, indices: &[i32]) -> i32 {
    if root.is_null() || result.is_null() { return 0; }
    let mut it = indices.iter().copied().take_while(|&t| t != -1);
    jobj_find_many(root, result, &mut it)
}

pub unsafe fn lb_8001204C(root: *mut HSD_JObj, result: *mut *mut HSD_JObj, indices: *mut u16, count: i32) -> i32 {
    if root.is_null() || result.is_null() || indices.is_null() || count == 0 { return 0; }
    let mut i = 0i32;
    let mut it = core::iter::from_fn(|| {
        if i < count { let v = *indices.add(i as usize) as i32; i += 1; Some(v) } else { None }
    });
    jobj_find_many(root, result, &mut it)
}

unsafe fn setImageFromPreloadedArchive(image_desc: *mut HSD_ImageDesc, entry_num: i16) -> *mut c_void {
    let image_ptr = ext::lbDvd_GetPreloadedArchive(entry_num as i32);
    (*image_desc).image_ptr = image_ptr;
    image_ptr
}

pub unsafe fn lb_800121FC(image_desc: *mut HSD_ImageDesc, width: i32, height: i32, format: u32, entry_num: i16) -> *mut HSD_ImageDesc {
    (*image_desc).width = width as u16;
    (*image_desc).height = height as u16;
    (*image_desc).mipmap = 0;
    (*image_desc).minLOD = 0.0;
    (*image_desc).maxLOD = 0.0;
    (*image_desc).format = format;
    hsd_assert((*image_desc).image_ptr.is_null(), "lbspdisplay.c:41 !image_desc->image_ptr");
    let buffer_size = ext::GXGetTexBufferSize((*image_desc).width, (*image_desc).height, (*image_desc).format, 0, 0);
    if entry_num == 0 || setImageFromPreloadedArchive(image_desc, entry_num).is_null() {
        (*image_desc).image_ptr = ext::HSD_MemAlloc(((buffer_size.wrapping_add(0x1F)) & !0x1F) as i32);
    }
    image_desc
}

pub unsafe fn lb_800122C8(image_desc: *mut HSD_ImageDesc, origx: u16, origy: u16, clear: bool) {
    ext::HSD_ImageDescCopyFromEFB(image_desc, origx, origy, clear as u8, 1);
}

#[inline]
fn f2u8(x: f32) -> u8 { (x as i32) as i8 as u8 }

pub unsafe fn lb_800122F0(img: *mut HSD_ImageDesc, tex: *mut _GXTexObj, factor: f32) {
    let color0 = _GXColor {
        r: f2u8(fmadds(-179.0, factor, 255.0)),
        g: f2u8(68.4 * factor),
        b: f2u8(60.8 * factor),
        a: 0,
    };
    let color1 = _GXColor {
        r: f2u8(150.0 * factor),
        g: f2u8(fmadds(-120.0, factor, 255.0)),
        b: f2u8(120.0 * factor),
        a: 0,
    };
    let color2 = _GXColor {
        r: f2u8(29.0 * factor),
        g: f2u8(26.099998 * factor),
        b: f2u8(fmadds(-231.8, factor, 255.0)),
        a: 0,
    };

    ext::GXInitTexObj(tex, (*img).image_ptr, (*img).width, (*img).height, (*img).format, GX_CLAMP, GX_CLAMP, (*img).mipmap as u8);
    ext::GXClearVtxDesc();
    ext::GXSetCullMode(GX_CULL_BACK);
    ext::GXSetNumTexGens(1);
    ext::GXSetZMode(0, GX_ALWAYS, 0);
    ext::GXLoadTexObj(tex, GX_TEXMAP0);
    ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_TEX_S, GX_RGBA6, 0);
    ext::GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
    ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_TEX0, GX_TEX_ST, GX_RGBA6, 0);
    ext::GXSetVtxDesc(GX_VA_TEX0, GX_DIRECT);

    if factor != 0.0 {
        ext::GXSetNumTevStages(3);
        ext::GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        ext::GXSetTevOrder(GX_TEVSTAGE1, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        ext::GXSetTevOrder(GX_TEVSTAGE2, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        ext::GXSetTevKColor(GX_KCOLOR0, color0);
        ext::GXSetTevKColor(GX_KCOLOR1, color1);
        ext::GXSetTevKColor(GX_KCOLOR2, color2);
        ext::GXSetTevKColorSel(GX_TEVSTAGE0, GX_TEV_KCSEL_K0);
        ext::GXSetTevKColorSel(GX_TEVSTAGE1, GX_TEV_KCSEL_K1);
        ext::GXSetTevKColorSel(GX_TEVSTAGE2, GX_TEV_KCSEL_K2);
        ext::GXSetTevSwapMode(GX_TEVSTAGE0, GX_TEV_SWAP1, GX_TEV_SWAP1);
        ext::GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_TEXC, GX_CC_KONST, GX_CC_ZERO);
        ext::GXSetTevColorOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1, GX_TEVPREV);
        ext::GXSetTevSwapMode(GX_TEVSTAGE1, GX_TEV_SWAP2, GX_TEV_SWAP2);
        ext::GXSetTevColorIn(GX_TEVSTAGE1, GX_CC_ZERO, GX_CC_TEXC, GX_CC_KONST, GX_CC_CPREV);
        ext::GXSetTevColorOp(GX_TEVSTAGE1, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1, GX_TEVPREV);
        ext::GXSetTevSwapMode(GX_TEVSTAGE2, GX_TEV_SWAP3, GX_TEV_SWAP3);
        ext::GXSetTevColorIn(GX_TEVSTAGE2, GX_CC_ZERO, GX_CC_TEXC, GX_CC_KONST, GX_CC_CPREV);
        ext::GXSetTevColorOp(GX_TEVSTAGE2, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1, GX_TEVPREV);
        ext::GXSetTevAlphaIn(GX_TEVSTAGE1, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO, GX_CA_APREV);
        ext::GXSetTevAlphaOp(GX_TEVSTAGE1, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1, GX_TEVPREV);
        ext::GXSetTevAlphaIn(GX_TEVSTAGE2, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO, GX_CA_APREV);
        ext::GXSetTevAlphaOp(GX_TEVSTAGE2, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1, GX_TEVPREV);
    } else {
        ext::GXSetNumTevStages(1);
        ext::GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        ext::GXSetTevSwapMode(GX_TEVSTAGE0, GX_TEV_SWAP0, GX_TEV_SWAP0);
        ext::GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_ZERO, GX_CC_ZERO, GX_CC_TEXC);
        ext::GXSetTevColorOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1, GX_TEVPREV);
    }

    ext::GXSetBlendMode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR);
    ext::GXSetColorUpdate(1);
    ext::GXSetAlphaCompare(GX_GEQUAL, 0, GX_AOP_OR, GX_GEQUAL, 0);
}

pub unsafe fn lb_8001271C(arg0: *mut _GXTexObj, x0: f32, arg2: f32, tex_width: f32, tex_height: f32, scale_x: f32, scale_y: f32) {
    let t = tex_width / (ext::GXGetTexObjWidth(arg0) as f32);
    let s = tex_height / (ext::GXGetTexObjHeight(arg0) as f32);

    ext::GXBegin(GX_QUADS, GX_VTXFMT0, 4);
    let w = tex_width * scale_x;
    let h = tex_height * scale_y;
    let y0 = -arg2;
    ext::GXPosition2f32(x0, y0);
    let x1 = x0 + w;
    let y_sum = arg2 + h;
    let zero = 0.0f32;
    ext::GXTexCoord2f32(0.0, zero);
    ext::GXPosition2f32(x1, y0);
    ext::GXTexCoord2f32(s, 0.0);
    let y1 = -y_sum;
    ext::GXPosition2f32(x1, y1);
    ext::GXTexCoord2f32(s, t);
    ext::GXPosition2f32(x0, y1);
    ext::GXTexCoord2f32(zero, t);
    // GXEnd(): no-op on this build
}

pub unsafe fn lb_8001285C(image_desc: *mut HSD_ImageDesc, tex_obj: *mut _GXTexObj) {
    ext::GXInitTexObj(tex_obj, (*image_desc).image_ptr, (*image_desc).width, (*image_desc).height,
                      (*image_desc).format, GX_CLAMP, GX_CLAMP, (*image_desc).mipmap as u8);
    ext::GXGetTexObjFmt(tex_obj);
    ext::GXClearVtxDesc();
    ext::GXSetCullMode(GX_CULL_BACK);
    ext::GXSetNumTexGens(1);
    ext::GXSetNumTevStages(1);
    ext::GXSetZMode(0, GX_ALWAYS, 0);
    ext::GXLoadTexObj(tex_obj, GX_TEXMAP0);
    ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_TEX_S, GX_RGBA6, 0);
    ext::GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
    ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_TEX0, GX_TEX_ST, GX_RGBA6, 0);
    ext::GXSetVtxDesc(GX_VA_TEX0, GX_DIRECT);
    ext::GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
    ext::GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_ZERO, GX_CC_ONE, GX_CC_TEXC);
    ext::GXSetTevColorOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1, GX_TEVPREV);
    ext::GXSetBlendMode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR);
}

unsafe fn setTevAlpha(alpha: u8) {
    let color = _GXColor { r: 0, g: 0, b: 0, a: alpha };
    ext::GXSetTevColor(GX_TEVREG0, color);
    ext::GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_TEXA, GX_CA_ZERO, GX_CA_A0, GX_CA_ZERO);
    ext::GXSetTevAlphaOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1, GX_TEVPREV);
}

pub unsafe fn lb_80012994(img: *mut HSD_ImageDesc, alpha: u8, blur_size: u8, x: f32, y: f32, scale_x: f32, scale_y: f32, color_factor: f32) {
    let mut tex: _GXTexObj = core::mem::zeroed();
    let w = (*img).width as f32;
    let h = (*img).height as f32;
    let off1 = (blur_size as f32) / 64.0;
    let off2 = 2.0 * off1;

    lb_800122F0(img, &mut tex, color_factor);

    let t: *mut _GXTexObj = &mut tex;
    setTevAlpha(alpha);
    lb_8001271C(t, x, y, w, h, scale_x, scale_y);

    setTevAlpha(0x7F);
    let x_p1 = x + off1;
    lb_8001271C(t, x_p1, y, w, h, scale_x, scale_y);

    setTevAlpha(0xA9);
    let x_m1 = x - off1;
    lb_8001271C(t, x_m1, y, w, h, scale_x, scale_y);

    setTevAlpha(0xBF);
    let y_p1 = y + off1;
    lb_8001271C(t, x, y_p1, w, h, scale_x, scale_y);

    setTevAlpha(0xCC);
    let y_m1 = y - off1;
    lb_8001271C(t, x, y_m1, w, h, scale_x, scale_y);

    setTevAlpha(0xD4);
    lb_8001271C(t, x_p1, y_p1, w, h, scale_x, scale_y);

    setTevAlpha(0xDA);
    lb_8001271C(t, x_m1, y_m1, w, h, scale_x, scale_y);

    setTevAlpha(0xDF);
    lb_8001271C(t, x_p1, y_m1, w, h, scale_x, scale_y);

    setTevAlpha(0xE2);
    lb_8001271C(t, x_m1, y_p1, w, h, scale_x, scale_y);

    setTevAlpha(0xE5);
    let x_p2 = x + off2;
    lb_8001271C(t, x_p2, y, w, h, scale_x, scale_y);

    setTevAlpha(0xE7);
    let x_m2 = x - off2;
    lb_8001271C(t, x_m2, y, w, h, scale_x, scale_y);

    setTevAlpha(0xE9);
    lb_8001271C(t, x_p2, y_p1, w, h, scale_x, scale_y);

    setTevAlpha(0xEB);
    lb_8001271C(t, x_m2, y_p1, w, h, scale_x, scale_y);

    setTevAlpha(0xEC);
    lb_8001271C(t, x_p2, y_m1, w, h, scale_x, scale_y);

    setTevAlpha(0xEE);
    lb_8001271C(t, x_m2, y_m1, w, h, scale_x, scale_y);

    setTevAlpha(0xEF);
    let y_p2 = y + off2;
    lb_8001271C(t, x, y_p2, w, h, scale_x, scale_y);

    setTevAlpha(0xF0);
    let y_m2 = y - off2;
    lb_8001271C(t, x, y_m2, w, h, scale_x, scale_y);

    setTevAlpha(0xF0);
    lb_8001271C(t, x_p1, y_p2, w, h, scale_x, scale_y);

    setTevAlpha(0xF1);
    lb_8001271C(t, x_p1, y_m2, w, h, scale_x, scale_y);

    setTevAlpha(0xF2);
    lb_8001271C(t, x_m1, y_p2, w, h, scale_x, scale_y);

    setTevAlpha(0xF2);
    lb_8001271C(t, x_m1, y_m2, w, h, scale_x, scale_y);

    ext::HSD_StateInvalidate(2);
}

static mut chan0: HSD_Chan = HSD_Chan {
    next: null_mut(),
    chan: GX_COLOR0,
    flags: 0,
    amb_color: _GXColor { r: 0, g: 0, b: 0, a: 0 },
    mat_color: _GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0 },
    enable: 0,
    amb_src: GX_SRC_REG,
    mat_src: GX_SRC_REG,
    light_mask: GX_LIGHT_NULL,
    diff_fn: GX_DF_CLAMP,
    attn_fn: GX_AF_NONE,
    aobj: null_mut(),
};

static mut chan1: HSD_Chan = HSD_Chan {
    next: null_mut(),
    chan: GX_ALPHA0,
    flags: 0,
    amb_color: _GXColor { r: 0, g: 0, b: 0, a: 0xFF },
    mat_color: _GXColor { r: 0, g: 0, b: 0, a: 0xFF },
    enable: 0,
    amb_src: GX_SRC_REG,
    mat_src: GX_SRC_REG,
    light_mask: GX_LIGHT_NULL,
    diff_fn: GX_DF_CLAMP,
    attn_fn: GX_AF_NONE,
    aobj: null_mut(),
};

pub unsafe fn fn_80013614(gobj: *mut HSD_GObj) {
    let data = (*gobj).user_data as *mut CameraBlurData;
    let mut view_mtx: Mtx = core::mem::zeroed();
    let mut tex_obj: _GXTexObj = core::mem::zeroed();

    if let Some(cb) = (*data).callback {
        cb(gobj);
    }

    let cobj = (*gobj).hsd_obj as *mut HSD_CObj;
    if (*data).mode == 1 {
        let tint_factor = (*data).tint_factor;
        let blur_size = (*data).blur_size;
        let base_alpha = (*data).base_alpha;
        let scale_y = (*data).scale_y;
        let scale_x = (*data).scale_x;
        let pos_y = (*data).pos_y;
        let pos_x = (*data).pos_x;
        let efb_copy = (*data).efb_copy;

        ext::HSD_CObjSetCurrent(cobj);
        ext::HSD_StateSetZMode(0, 7, 0);
        ext::GXSetCurrentMtx(0);
        ext::HSD_CObjGetViewingMtx(ext::HSD_CObjGetCurrent(), view_mtx.as_mut_ptr() as *mut [f32; 4]);
        ext::GXLoadPosMtxImm(view_mtx.as_mut_ptr() as *mut [f32; 4], 0);
        ext::GXSetTexCoordGen2(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, 0x3C, 0, 0x7D);
        ext::HSD_SetupChannel(&raw mut chan0);
        ext::HSD_SetupChannel(&raw mut chan1);
        ext::HSD_StateSetNumChans(1);

        if tint_factor != 0.0 {
            ext::GXSetScissor(0, 110, 640, 290);
        }

        lb_80012994(efb_copy, base_alpha, blur_size, pos_x, pos_y, scale_x, scale_y, tint_factor);
    } else {
        let base_alpha = (*data).base_alpha;
        let scale_y = (*data).scale_y;
        let scale_x = (*data).scale_x;
        let pos_y = (*data).pos_y;
        let pos_x = (*data).pos_x;
        let efb_copy = (*data).efb_copy;

        ext::HSD_CObjSetCurrent(cobj);
        ext::HSD_StateSetZMode(0, 7, 0);
        ext::GXSetCurrentMtx(0);
        ext::HSD_CObjGetViewingMtx(ext::HSD_CObjGetCurrent(), view_mtx.as_mut_ptr() as *mut [f32; 4]);
        ext::GXLoadPosMtxImm(view_mtx.as_mut_ptr() as *mut [f32; 4], 0);
        ext::GXSetTexCoordGen2(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, 0x3C, 0, 0x7D);
        ext::HSD_SetupChannel(&raw mut chan0);
        ext::HSD_SetupChannel(&raw mut chan1);
        ext::HSD_StateSetNumChans(1);

        let width = (*efb_copy).width;
        let height = (*efb_copy).height;
        lb_8001285C(efb_copy, &mut tex_obj);

        setTevAlpha(base_alpha);
        lb_8001271C(&mut tex_obj, pos_x, pos_y, width as f32, height as f32, scale_x, scale_y);
        ext::HSD_StateInvalidate(2);
    }
}

/// C-ABI render callback (`GObj_RenderFunc` view of fn_80013614, which the C casts from `void(*)(HSD_GObj*)`).
unsafe extern "C" fn fn_80013614_cb(gobj: *mut HSD_GObj, _pass: i32) {
    fn_80013614(gobj)
}

pub unsafe fn fn_800138AC(ptr: *mut c_void) {
    ext::HSD_Free(ptr);
}
unsafe extern "C" fn fn_800138AC_cb(ptr: *mut c_void) {
    fn_800138AC(ptr)
}

pub unsafe fn lb_800138CC(gobj: *mut HSD_GObj, blur_callback: HSD_GObjEvent) {
    let data = (*gobj).user_data as *mut CameraBlurData;
    (*data).callback = blur_callback;
}

pub unsafe fn lb_800138D8(gobj: *mut HSD_GObj, size: i8) {
    let data = (*gobj).user_data as *mut CameraBlurData;
    (*data).mode = 1;
    (*data).blur_size = size as u8;
}

const lb_803B72A8: Vec = Vec { x: 0.0, y: 0.0, z: 1.0 };
const lb_803B72B4: Vec = Vec { x: 0.0, y: 0.0, z: 0.0 };

pub unsafe fn lb_800138EC(img: *mut HSD_ImageDesc, render_func: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>,
                          prio: i32, alpha: i8, pos_x: f32, pos_y: f32, scale_x: f32, scale_y: f32) -> *mut HSD_GObj {
    let zero = 0.0f32;
    let far = 2.0f32;
    let ortho_top = zero;
    let ortho_bot = -480.0f32;
    let ortho_left = zero;
    let ortho_right = 640.0f32;

    let mut eye: Vec = lb_803B72A8;
    let roll = zero;
    let mut interest: Vec = lb_803B72B4;

    let gobj = ext::GObj_Create(14, 15, 0);
    let cobj = ext::HSD_CObjAlloc();

    let mut viewport = _HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 480 };
    let mut scissor = _Scissor { left: 0, right: 640, top: 0, bottom: 480 };

    ext::HSD_CObjSetProjectionType(cobj, 3);
    ext::HSD_CObjSetViewport(cobj, &mut viewport);
    ext::HSD_CObjSetScissor(cobj, &mut scissor);
    ext::HSD_CObjSetEyePosition(cobj, &mut eye);
    ext::HSD_CObjSetInterest(cobj, &mut interest);
    ext::HSD_CObjSetRoll(cobj, roll);
    ext::HSD_CObjSetNear(cobj, zero);
    ext::HSD_CObjSetFar(cobj, far);
    ext::HSD_CObjSetOrtho(cobj, ortho_top, ortho_bot, ortho_left, ortho_right);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);

    let data = ext::HSD_MemAlloc(core::mem::size_of::<CameraBlurData>() as i32) as *mut CameraBlurData;
    (*data).pos_x = pos_x;
    (*data).pos_y = pos_y;
    (*data).scale_x = scale_x;
    (*data).scale_y = scale_y;
    (*data).base_alpha = alpha as u8;
    (*data).mode = 0;
    (*data).efb_copy = img;
    (*data).callback = None;
    ext::GObj_InitUserData(gobj, 0, Some(fn_800138AC_cb), data as *mut c_void);

    match render_func {
        None => ext::GObj_SetupGXLinkMax(gobj, Some(fn_80013614_cb), prio as u32),
        Some(_) => ext::GObj_SetupGXLinkMax(gobj, render_func, prio as u32),
    }
    gobj
}

pub unsafe fn lb_80013B14(desc: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj {
    let cobj = ext::HSD_CObjLoadDesc(desc as *mut HSD_CObjDesc);
    let mut scissor: _Scissor = core::mem::zeroed();

    // TARGET_PC: authored aspect
    if ext::HSD_CObjGetProjectionType(cobj) == 1 && ext::HSD_CObjGetAuthoredAspect(cobj) == 1.18 {
        ext::HSD_CObjSetAspect(cobj, 1.2173333);
    }
    ext::HSD_CObjGetScissor(cobj, &mut scissor);
    if scissor.right > 640 { scissor.right = 640; }
    if scissor.bottom > 480 { scissor.bottom = 480; }
    ext::HSD_CObjSetScissor(cobj, &mut scissor);
    cobj
}
