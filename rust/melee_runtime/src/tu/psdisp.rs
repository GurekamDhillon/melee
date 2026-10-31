//! Native particle display implementation from `psdisp.c`.
//! Contraction sites mirror shipping Clang (170 f32 sites).
#![allow(non_snake_case, non_upper_case_globals, unused_variables, unused_mut, unused_assignments, unused_parens, static_mut_refs, dead_code)]
use melee_types::*;
use melee_core::fp::fmadds;
use crate::gekko::{sinf,cosf,tanf,sqrtf};
use ext::*;
// BEGIN GENERATED EXT (rust/melee_runtime/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn GXBegin(a0: u32, a1: u32, a2: u16);
    fn GXClearVtxDesc();
    fn GXColor4u8(a0: u8, a1: u8, a2: u8, a3: u8);
    fn GXEnableTexOffsets(a0: u32, a1: u8, a2: u8);
    fn GXGetProjectionv(a0: *mut f32);
    fn GXInitTexObj(a0: *mut _GXTexObj, a1: *mut c_void, a2: u16, a3: u16, a4: u32, a5: u32, a6: u32, a7: u8);
    fn GXInitTexObjCI(a0: *mut _GXTexObj, a1: *mut c_void, a2: u16, a3: u16, a4: u32, a5: u32, a6: u32, a7: u8, a8: u32);
    fn GXInitTexObjLOD(a0: *mut _GXTexObj, a1: u32, a2: u32, a3: f32, a4: f32, a5: f32, a6: u8, a7: u8, a8: u32);
    fn GXInitTlutObj(a0: *mut _GXTlutObj, a1: *mut c_void, a2: u32, a3: u16);
    fn GXLoadPosMtxImm(a0: *mut c_void, a1: u32);
    fn GXLoadTexMtxImm(a0: *mut c_void, a1: u32, a2: u32);
    fn GXLoadTexObj(a0: *mut _GXTexObj, a1: u32);
    fn GXLoadTlut(a0: *mut _GXTlutObj, a1: u32);
    fn GXPosition3f32(a0: f32, a1: f32, a2: f32);
    fn GXSetAlphaCompare(a0: u32, a1: u8, a2: u32, a3: u32, a4: u8);
    fn GXSetArray(a0: u32, a1: *const c_void, a2: u8);
    fn GXSetBlendMode(a0: u32, a1: u32, a2: u32, a3: u32);
    fn GXSetChanAmbColor(a0: u32, a1: _GXColor);
    fn GXSetChanCtrl(a0: u32, a1: u8, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32);
    fn GXSetChanMatColor(a0: u32, a1: _GXColor);
    fn GXSetCullMode(a0: u32);
    fn GXSetCurrentMtx(a0: u32);
    fn GXSetLineWidth(a0: u8, a1: u32);
    fn GXSetNumChans(a0: u8);
    fn GXSetPointSize(a0: u8, a1: u32);
    fn GXSetTevColor(a0: u32, a1: _GXColor);
    fn GXSetTexCoordGen2(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32);
    fn GXSetVtxAttrFmt(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8);
    fn GXSetVtxDesc(a0: u32, a1: u32);
    fn GXSetZCompLoc(a0: u8);
    fn GXSetZMode(a0: u8, a1: u32, a2: u8);
    fn GXTexCoord1x8(a0: u8);
    fn GXTexCoord2f32(a0: f32, a1: f32);
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj;
    fn HSD_CObjGetViewingMtx(a0: *mut HSD_CObj, a1: *mut c_void);
    fn HSD_FogSet(a0: *mut HSD_Fog);
    fn HSD_LObjGetActiveByID(a0: u32) -> *mut HSD_LObj;
    fn HSD_LObjGetLightMaskAttnFunc() -> i32;
    fn HSD_LObjGetLightMaskDiffuse() -> u32;
    fn HSD_MtxGetScale(a0: *mut c_void, a1: *mut Vec) = { let m = (a0 as *const [[f32; 4]; 3]).read(); let r = crate::hsd_mtx::get_scale(&m); (*a1).x = r.x; (*a1).y = r.y; (*a1).z = r.z; };
    fn HSD_MtxSRT(a0: *mut c_void, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec, a4: *mut Vec) = { let v = |x: *mut Vec| crate::mtx::Vec3 { x: (*x).x, y: (*x).y, z: (*x).z }; let ps = if a4.is_null() { None } else { Some(v(a4)) }; (a0 as *mut [[f32; 4]; 3]).write(crate::hsd_mtx::srt(&v(a1), &v(a2), &v(a3), ps.as_ref())); };
    fn HSD_MulColor(a0: *mut _GXColor, a1: *mut _GXColor, a2: *mut _GXColor) = { let v = |x: *mut GXColor| crate::hsd_util::GxColor { r: (*x).r, g: (*x).g, b: (*x).b, a: (*x).a }; let r = crate::hsd_util::mul_color(&v(a0), &v(a1)); *a2 = GXColor { r: r.r, g: r.g, b: r.b, a: r.a }; };
    fn HSD_StateInvalidate(a0: i32);
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void) = crate::tu::mtx_pc::raw::PSMTXConcat(a0 as _, a1 as _, a2 as _);
    fn PSMTXCopy(a0: *mut c_void, a1: *mut c_void) = crate::tu::mtx_pc::raw::PSMTXCopy(a0 as _, a1 as _);
    fn PSMTXIdentity(a0: *mut c_void) = crate::tu::mtx_pc::raw::PSMTXIdentity(a0 as _);
    fn PSMTXInverse(a0: *mut c_void, a1: *mut c_void) -> u32 = { let s = (a0 as *const [[f32; 4]; 3]).read(); if let Some(m) = crate::mtx::inverse(&s) { (a1 as *mut [[f32; 4]; 3]).write(m); 1 } else { 0 } };
    fn PSMTXRotAxisRad(a0: *mut c_void, a1: *mut Vec, a2: f32) = { let v = crate::mtx::Vec3 { x: (*a1).x, y: (*a1).y, z: (*a1).z }; (a0 as *mut [[f32; 4]; 3]).write(crate::mtx::rot_axis_rad(&v, a2)); };
    fn PSMTXScale(a0: *mut c_void, a1: f32, a2: f32, a3: f32) = (a0 as *mut [[f32; 4]; 3]).write(crate::mtx::scale(a1, a2, a3));
    fn atan2f(a0: f32, a1: f32) -> f32 = crate::lbtrigf::atan2f(a0, a1);
    fn psSetupTev(a0: *mut HSD_Particle) = crate::tu::psdisptev::psSetupTev(a0);
    fn psSetupTevCommon() = crate::tu::psdisptev::psSetupTevCommon();
    fn psSetupTevInvalidState() = crate::tu::psdisptev::psSetupTevInvalidState();
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::particle::hsd_804D0908;
    pub static mut psNumCmdList: [*mut *mut _HSD_PSFormGroup; 65] = unsafe { core::mem::zeroed() };
    pub static mut psTexGroupArray: [*mut *mut _HSD_PSTexGroup; 65] = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT
type MP = *mut [f32; 4];
unsafe fn atan2f(y: f32, x: f32) -> f32 { ext::atan2f(y, x) }
unsafe fn memset(dst: *mut core::ffi::c_void, value: i32, n: usize) -> *mut core::ffi::c_void {
    core::ptr::write_bytes(dst.cast::<u8>(), value as u8, n);
    dst
}
/// `__assert` (debug.c): the C build does not return.
unsafe fn __assert(file: *const core::ffi::c_char, line: u32, cond: *const core::ffi::c_char) -> ! {
    let _ = (file, cond);
    panic!("psdisp.c __assert line {line}")
}
/// `OSReport(fmt)` (variadic; a settable lbext hook).
unsafe fn OSReport(fmt: *mut core::ffi::c_char) {
    crate::tu::lbext::OSReport(&core::ffi::CStr::from_ptr(fmt).to_string_lossy(), &[]);
}
// The generated hooks take `Mtx` parameters as `void*`; the port keeps `MtxPtr`.
unsafe fn PSMTXIdentity(m: MP) { ext::PSMTXIdentity(m.cast()) }
unsafe fn PSMTXCopy(src: MP, dst: MP) { ext::PSMTXCopy(src.cast(), dst.cast()) }
unsafe fn PSMTXConcat(a: MP, b: MP, dst: MP) { ext::PSMTXConcat(a.cast(), b.cast(), dst.cast()) }
unsafe fn PSMTXScale(dst: MP, x: f32, y: f32, z: f32) { ext::PSMTXScale(dst.cast(), x, y, z) }
unsafe fn PSMTXRotAxisRad(dst: MP, v: *mut Vec3, r: f32) { ext::PSMTXRotAxisRad(dst.cast(), v.cast(), r) }
unsafe fn PSMTXInverse(src: MP, dst: MP) -> u32 { ext::PSMTXInverse(src.cast(), dst.cast()) }
unsafe fn HSD_CObjGetViewingMtx(c: *mut HSD_CObj, m: MP) { ext::HSD_CObjGetViewingMtx(c, m.cast()) }
unsafe fn GXLoadPosMtxImm(m: MP, id: u32) { ext::GXLoadPosMtxImm(m.cast(), id) }
unsafe fn GXLoadTexMtxImm(m: MP, id: u32, ty: GXTexMtxType) { ext::GXLoadTexMtxImm(m.cast(), id, ty as _) }
unsafe fn HSD_MtxGetScale(m: MP, v: *mut Vec3) { ext::HSD_MtxGetScale(m.cast(), v.cast()) }
unsafe fn HSD_MtxSRT(m: MP, s: *mut Vec3, r: *mut Vec3, t: *mut Vec3, p: *mut Vec3) { ext::HSD_MtxSRT(m.cast(), s.cast(), r.cast(), t.cast(), p.cast()) }

pub static mut HSD_PSDisp_803B9628: psdisp_Mtx = psdisp_Mtx {
    mtx: [
        [1.0f32, 0.0f32, 0.0f32, 0.0f32],
        [0.0f32, 1.0f32, 0.0f32, 0.0f32],
        [0.0f32, 0.0f32, 1.0f32, 0.0f32],
    ],
};
pub static mut HSD_PSDisp_8040C340: [u8; 32] = [
    0 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
    1 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
    0 as i32 as i32 as u8,
];
pub static mut HSD_PSDisp_8040C360: [u8; 16] = [
    0 as i32 as i32 as u8,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];
pub static mut psFrameNum: u8 = 0x7b as i32 as u8;
pub static mut vmtx: Mtx = [[0.; 4]; 3];
pub static mut rvmtx: Mtx = [[0.; 4]; 3];
pub static mut prj: [f32; 7] = [0.; 7];
pub static mut pvmtx: Mtx = [[0.; 4]; 3];
pub static mut particle_list: [*mut HSD_Particle; 17] = [::core::ptr::null::<HSD_Particle>()
    as *mut HSD_Particle; 17];
pub static mut HSD_PSDisp_804D7908: *mut HSD_Fog = ::core::ptr::null::<HSD_Fog>()
    as *mut HSD_Fog;
pub static mut prevPointSize: s32 = 0;
pub static mut prevLineWidth: s32 = 0;
pub static mut HSD_PSDisp_804D7914: f32 = 0.;
pub static mut HSD_PSDisp_804D7918: f32 = 0.;
pub static mut HSD_PSDisp_804D791C: f32 = 0.;
pub static mut HSD_PSDisp_804D7920: f32 = 0.;
pub static mut HSD_PSDisp_804D7924: f32 = 0.;
pub static mut HSD_PSDisp_804D7928: f32 = 0.;
pub static mut HSD_PSDisp_804D792C: s32 = 0;
pub static mut prevChanCtrl: s32 = 0;
pub static mut prevChanMat: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
pub static mut prevChanAmb: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
pub static mut prevColorPrim: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
pub static mut prevColorEnv: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
pub static mut prevColorMat: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
pub static mut HSD_PSDisp_804D7948: [s32; 2] = [0; 2];

pub unsafe fn setVtxDesc(mut fmt: s32) {
    melee_core::ct!("setVtxDesc");
    GXClearVtxDesc();
    match fmt {
        0 => {
            GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
            GXSetVtxDesc(GX_VA_TEX0, GX_INDEX8);
            return;
        }
        1 => {
            GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
            return;
        }
        2 => {
            GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
            GXSetVtxDesc(GX_VA_CLR0, GX_DIRECT);
            GXSetVtxDesc(GX_VA_TEX0, GX_INDEX8);
            return;
        }
        3 => {
            GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
            GXSetVtxDesc(GX_VA_CLR0, GX_DIRECT);
            return;
        }
        4 => {
            GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
            GXSetVtxDesc(GX_VA_TEX0, GX_DIRECT);
            return;
        }
        5 => {
            GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
            GXSetVtxDesc(GX_VA_CLR0, GX_DIRECT);
            GXSetVtxDesc(GX_VA_TEX0, GX_DIRECT);
            return;
        }
        _ => {}
    };
}
pub unsafe fn calcTornadoLastPos(
    mut pp: *mut HSD_Particle,
    mut x: *mut f32,
    mut y: *mut f32,
    mut z: *mut f32,
) {
    melee_core::ct!("calcTornadoLastPos");
    let mut radius: f32 = 0.;
    let mut px: f32 = 0.;
    let mut py: f32 = 0.;
    let mut pz: f32 = 0.;
    let mut sina: f32 = 0.;
    let mut sinb: f32 = 0.;
    let mut cosa: f32 = 0.;
    let mut cosb: f32 = 0.;
    let mut vx0: f32 = 0.;
    let mut vz0: f32 = 0.;
    let mut gp: *mut HSD_Generator = ::core::ptr::null_mut::<HSD_Generator>();
    gp = (*pp).gen as *mut HSD_Generator;
    if gp.is_null() {
        *x = (*gp).pos.x;
        *y = (*gp).pos.y;
        *z = (*gp).pos.z;
        return;
    }
    sina = sinf((*pp).grav) as f32;
    sinb = sinf((*pp).fric) as f32;
    cosa = cosf((*pp).grav) as f32;
    cosb = cosf((*pp).fric) as f32;
    vz0 = (*pp).vel.z - (*gp).aux.tornado.vel;
    vx0 = (*pp).vel.x - (*gp).grav;
    radius = if (*gp).radius < 0 as i32 as f32 {
        -(*gp).radius
    } else {
        (*gp).radius
    };
    radius = fmadds(
        vz0 as f32,
        tanf(
            if (*gp).angle < 0 as i32 as f32 {
                -(*gp).angle
            } else {
                (*gp).angle
            },
        ) as f32,
        radius as f32,
    ) as f32;
    radius *= (*pp).vel.y;
    px = (radius as f32 * cosf(vx0 as f32)) as f32;
    py = (radius as f32 * sinf(vx0 as f32)) as f32;
    pz = vz0;
    *x = fmadds(
        px as f32,
        cosb as f32,
        pz as f32 * sinb as f32,
    ) as f32 + (*gp).pos.x;
    *y = fmadds(
        pz as f32 * sina as f32,
        cosb as f32,
        fmadds(
            -(px as f32) * sina as f32,
            sinb as f32,
            py as f32 * cosa as f32,
        ),
    ) as f32 + (*gp).pos.y;
    *z = fmadds(
        pz as f32 * cosa as f32,
        cosb as f32,
        fmadds(
            -(px as f32) * cosa as f32,
            sinb as f32,
            -(py as f32 * sina as f32),
        ),
    ) as f32 + (*gp).pos.z;
}
pub unsafe fn getColorPrimEnv(
    mut pp: *mut HSD_Particle,
    mut primCol: *mut GXColor,
    mut envCol: *mut GXColor,
) {
    melee_core::ct!("getColorPrimEnv");
    if (*pp).primColCount != 0 {
        let mut scale: i32 = 65536 as i32
            * (*pp).primColRemain as i32
            / (*pp).primColCount as i32;
        (*primCol).r = ((((*pp).primColTarget.r as i32)
            << 16 as i32)
            + ((*pp).primCol.r as i32
                - (*pp).primColTarget.r as i32) * scale
            >> 16 as i32) as i32 as u8;
        (*primCol).g = ((((*pp).primColTarget.g as i32)
            << 16 as i32)
            + ((*pp).primCol.g as i32
                - (*pp).primColTarget.g as i32) * scale
            >> 16 as i32) as i32 as u8;
        (*primCol).b = ((((*pp).primColTarget.b as i32)
            << 16 as i32)
            + ((*pp).primCol.b as i32
                - (*pp).primColTarget.b as i32) * scale
            >> 16 as i32) as i32 as u8;
        (*primCol).a = ((((*pp).primColTarget.a as i32)
            << 16 as i32)
            + ((*pp).primCol.a as i32
                - (*pp).primColTarget.a as i32) * scale
            >> 16 as i32) as i32 as u8;
    } else {
        *primCol = (*pp).primCol;
    }
    if (*pp).envColCount != 0 {
        let mut scale_0: i32 = 65536 as i32
            * (*pp).envColRemain as i32
            / (*pp).envColCount as i32;
        (*envCol).r = ((((*pp).envColTarget.r as i32)
            << 16 as i32)
            + ((*pp).envCol.r as i32
                - (*pp).envColTarget.r as i32) * scale_0
            >> 16 as i32) as i32 as u8;
        (*envCol).g = ((((*pp).envColTarget.g as i32)
            << 16 as i32)
            + ((*pp).envCol.g as i32
                - (*pp).envColTarget.g as i32) * scale_0
            >> 16 as i32) as i32 as u8;
        (*envCol).b = ((((*pp).envColTarget.b as i32)
            << 16 as i32)
            + ((*pp).envCol.b as i32
                - (*pp).envColTarget.b as i32) * scale_0
            >> 16 as i32) as i32 as u8;
        (*envCol).a = ((((*pp).envColTarget.a as i32)
            << 16 as i32)
            + ((*pp).envCol.a as i32
                - (*pp).envColTarget.a as i32) * scale_0
            >> 16 as i32) as i32 as u8;
    } else {
        *envCol = (*pp).envCol;
    };
}
pub unsafe fn getColorMatAmb(
    mut pp: *mut HSD_Particle,
    mut matCol: *mut GXColor,
    mut ambCol: *mut GXColor,
) {
    melee_core::ct!("getColorMatAmb");
    if (*pp).matColCount != 0 {
        let mut scale: i32 = 65536 as i32
            * (*pp).matColRemain as i32
            / (*pp).matColCount as i32;
        (*matCol).b = ((((*pp).matRGBTarget as i32)
            << 16 as i32)
            + ((*pp).matRGB as i32
                - (*pp).matRGBTarget as i32) * scale
            >> 16 as i32) as i32 as u8;
        (*matCol).g = (*matCol).b;
        (*matCol).r = (*matCol).g;
        (*matCol).a = ((((*pp).matATarget as i32)
            << 16 as i32)
            + ((*pp).matA as i32 - (*pp).matATarget as i32)
                * scale >> 16 as i32) as i32 as u8;
    } else {
        (*matCol).b = (*pp).matRGB;
        (*matCol).g = (*matCol).b;
        (*matCol).r = (*matCol).g;
        (*matCol).a = (*pp).matA;
    }
    if (*pp).ambColCount != 0 {
        let mut scale_0: i32 = 65536 as i32
            * (*pp).ambColRemain as i32
            / (*pp).ambColCount as i32;
        (*ambCol).b = ((((*pp).ambRGBTarget as i32)
            << 16 as i32)
            + ((*pp).ambRGB as i32
                - (*pp).ambRGBTarget as i32) * scale_0
            >> 16 as i32) as i32 as u8;
        (*ambCol).g = (*ambCol).b;
        (*ambCol).r = (*ambCol).g;
        (*ambCol).a = ((((*pp).ambATarget as i32)
            << 16 as i32)
            + ((*pp).ambA as i32 - (*pp).ambATarget as i32)
                * scale_0 >> 16 as i32) as i32 as u8;
    } else {
        (*ambCol).b = (*pp).ambRGB;
        (*ambCol).g = (*ambCol).b;
        (*ambCol).r = (*ambCol).g;
        (*ambCol).a = (*pp).ambA;
    };
}

pub unsafe fn getClrTrail(mut pp: *mut HSD_Particle, mut color: *mut GXColor) {
    melee_core::ct!("getClrTrail");
    let mut env_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    match (*pp).kind
        & (DispLighting as i32 | PrimEnv as i32) as u32
    {
        0 | 2147483648 => {
            getColorPrimEnv(pp, color, &raw mut env_color);
        }
        128 | 2147483776 => {
            (*color).a = 0xff as i32 as u8;
            (*color).b = (*color).a;
            (*color).g = (*color).b;
            (*color).r = (*color).g;
        }
        _ => {}
    };
}

pub unsafe fn psSetColor(mut color: *mut GXColor, mut value: u8) {
    melee_core::ct!("psSetColor");
    (*color).r = value;
    (*color).g = value;
    (*color).b = value;
    (*color).a = value;
}

pub unsafe fn psSetupVtxFormat(
    mut format: GXVtxFmt,
    mut has_color: i32,
    mut has_texture: i32,
    mut texture_type: GXCompType,
) {
    melee_core::ct!("psSetupVtxFormat");
    GXSetVtxAttrFmt(format, GX_VA_POS, GX_TEX_ST, GX_RGBA6, 0 as i32 as u8);
    if has_color != 0 {
        GXSetVtxAttrFmt(format, GX_VA_CLR0, GX_TEX_ST, GX_RGBA8, 0 as i32 as u8);
    }
    if has_texture != 0 {
        GXSetVtxAttrFmt(format, GX_VA_TEX0, GX_TEX_ST, texture_type, 0 as i32 as u8);
    }
}

pub unsafe fn setupChanCtrl(mut pp: *mut HSD_Particle) {
    melee_core::ct!("setupChanCtrl");
    let mut chan_state: u32 = (*pp).kind
        & (DispLighting as i32 | Trail as i32) as u32;
    if chan_state != prevChanCtrl as u32 {
        prevChanCtrl = chan_state as s32;
        GXSetNumChans(1 as i32 as u8);
        match prevChanCtrl {
            1048576 => {
                GXSetChanCtrl(
                    GX_COLOR0A0,
                    0,
                    GX_SRC_VTX,
                    GX_SRC_VTX,
                    0 as u32,
                    GX_DF_NONE,
                    GX_AF_NONE,
                );
            }
            -2147483648 => {
                GXSetChanCtrl(
                    GX_COLOR0,
                    1,
                    GX_SRC_REG,
                    GX_SRC_REG,
                    HSD_LObjGetLightMaskDiffuse() as u32,
                    GX_DF_NONE,
                    (if HSD_LObjGetLightMaskAttnFunc() != 0 {
                        GX_AF_SPOT as i32
                    } else {
                        GX_AF_NONE as i32
                    }) as GXAttnFn,
                );
                GXSetChanCtrl(
                    GX_ALPHA0,
                    0,
                    GX_SRC_REG,
                    GX_SRC_REG,
                    0 as u32,
                    GX_DF_NONE,
                    GX_AF_NONE,
                );
            }
            -2146435072 => {
                GXSetChanCtrl(
                    GX_COLOR0,
                    1,
                    GX_SRC_REG,
                    GX_SRC_REG,
                    HSD_LObjGetLightMaskDiffuse() as u32,
                    GX_DF_NONE,
                    (if HSD_LObjGetLightMaskAttnFunc() != 0 {
                        GX_AF_SPOT as i32
                    } else {
                        GX_AF_NONE as i32
                    }) as GXAttnFn,
                );
                GXSetChanCtrl(
                    GX_ALPHA0,
                    0,
                    GX_SRC_VTX,
                    GX_SRC_VTX,
                    0 as u32,
                    GX_DF_NONE,
                    GX_AF_NONE,
                );
            }
            _ => {
                GXSetChanCtrl(
                    GX_COLOR0A0,
                    0,
                    GX_SRC_REG,
                    GX_SRC_REG,
                    0 as u32,
                    GX_DF_NONE,
                    GX_AF_NONE,
                );
            }
        }
    }
}

pub unsafe fn setupChanReg(mut pp: *mut HSD_Particle) {
    melee_core::ct!("setupChanReg");
    let mut prim_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut amb_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut mat_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut lobj: *mut HSD_LObj = ::core::ptr::null_mut::<HSD_LObj>();
    if (*pp).kind & DispLighting as i32 as u32 != 0 {
        getColorMatAmb(pp, &raw mut mat_color, &raw mut amb_color);
        if (*pp).kind & PrimEnv as i32 as u32 != 0 {
            prim_color.b = 0xff as i32 as u8;
            prim_color.g = prim_color.b;
            prim_color.r = prim_color.g;
        } else {
            getColorPrimEnv(pp, &raw mut prim_color, &raw mut mat_color);
            amb_color.r = (amb_color.r as i32
                * prim_color.r as i32 >> 8 as i32) as i32 as u8;
            amb_color.g = (amb_color.g as i32
                * prim_color.g as i32 >> 8 as i32) as i32 as u8;
            amb_color.b = (amb_color.b as i32
                * prim_color.b as i32 >> 8 as i32) as i32 as u8;
        }
        if prim_color.r as i32 != prevChanMat.r as i32
            || prim_color.g as i32 != prevChanMat.g as i32
            || prim_color.b as i32 != prevChanMat.b as i32
        {
            prevChanMat = prim_color;
            GXSetChanMatColor(GX_COLOR0, prevChanMat);
        }
        lobj = HSD_LObjGetActiveByID(GX_MAX_LIGHT);
        if !lobj.is_null() {
            HSD_MulColor(&raw mut amb_color, &raw mut (*lobj).color, &raw mut amb_color);
        } else {
            amb_color.b = 0 as i32 as u8;
            amb_color.g = amb_color.b;
            amb_color.r = amb_color.g;
        }
        if amb_color.r as i32 != prevChanAmb.r as i32
            || amb_color.g as i32 != prevChanAmb.g as i32
            || amb_color.b as i32 != prevChanAmb.b as i32
        {
            prevChanAmb = amb_color;
            GXSetChanAmbColor(GX_COLOR0, prevChanAmb);
        }
    }
}

pub unsafe fn setupTevReg(mut pp: *mut HSD_Particle) {
    melee_core::ct!("setupTevReg");
    let mut prim_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut env_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut mat_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut amb_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    getColorPrimEnv(pp, &raw mut prim_color, &raw mut env_color);
    if (*pp).kind & PrimEnv as i32 as u32 != 0
        || (*pp).kind & DispLighting as i32 as u32 == 0
            && (*pp).kind & Trail as i32 as u32 == 0
    {
        if prevColorPrim.r as i32 != prim_color.r as i32
            || prevColorPrim.g as i32
                != prim_color.g as i32
            || prevColorPrim.b as i32
                != prim_color.b as i32
            || prevColorPrim.a as i32
                != prim_color.a as i32
        {
            prevColorPrim = prim_color;
            GXSetTevColor(GX_TEVREG0, prevColorPrim);
        }
        if (*pp).kind & PrimEnv as i32 as u32 != 0 {
            if prevColorEnv.r as i32 != env_color.r as i32
                || prevColorEnv.g as i32
                    != env_color.g as i32
                || prevColorEnv.b as i32
                    != env_color.b as i32
                || prevColorEnv.a as i32
                    != env_color.a as i32
            {
                prevColorEnv = env_color;
                GXSetTevColor(GX_TEVREG1, prevColorEnv);
            }
        } else if prevColorEnv.r as i32 != 0 as i32
            || prevColorEnv.g as i32 != 0 as i32
            || prevColorEnv.b as i32 != 0 as i32
            || prevColorEnv.a as i32 != 0 as i32
        {
            prevColorEnv.a = 0 as i32 as u8;
            prevColorEnv.b = prevColorEnv.a;
            prevColorEnv.g = prevColorEnv.b;
            prevColorEnv.r = prevColorEnv.g;
            GXSetTevColor(GX_TEVREG1, prevColorEnv);
        }
    }
    if (*pp).kind & DispLighting as i32 as u32 != 0 {
        getColorMatAmb(pp, &raw mut mat_color, &raw mut amb_color);
        if (*pp).kind & PrimEnv as i32 as u32 != 0 {
            if prevColorMat.r as i32 != mat_color.r as i32
                || prevColorMat.g as i32
                    != mat_color.g as i32
                || prevColorMat.b as i32
                    != mat_color.b as i32
                || prevColorMat.a as i32
                    != mat_color.a as i32
            {
                prevColorMat = mat_color;
                GXSetTevColor(GX_TEVREG2, prevColorMat);
            }
        } else {
            mat_color.a = (mat_color.a as i32
                * prim_color.a as i32 >> 8 as i32) as i32 as u8;
            if prevColorMat.r as i32 != mat_color.r as i32
                || prevColorMat.g as i32
                    != mat_color.g as i32
                || prevColorMat.b as i32
                    != mat_color.b as i32
                || prevColorMat.a as i32
                    != mat_color.a as i32
            {
                prevColorMat = mat_color;
                GXSetTevColor(GX_TEVREG2, prevColorMat);
            }
        }
    }
}

pub unsafe fn particleSort(
    mut arg0: s32,
    mut arg1: u8,
    mut arg2: *mut *mut HSD_Particle,
    mut arg3: *mut *mut HSD_Particle,
) -> *mut HSD_Particle {
    melee_core::ct!("particleSort");
    let mut buckets: [psdisp_ParticleSortBucket; 16] = [psdisp_ParticleSortBucket {
        head: ::core::ptr::null_mut::<HSD_Particle>(),
        tail: ::core::ptr::null_mut::<HSD_Particle>(),
    }; 16];
    let mut new_var: *mut *mut HSD_Particle = ::core::ptr::null_mut::<
        *mut HSD_Particle,
    >();
    let mut var_r28: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    let mut var_r3: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    let mut var_r4: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    let mut var_r5: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    let mut var_r7: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    let mut temp_r29: *mut *mut HSD_Particle = ::core::ptr::null_mut::<
        *mut HSD_Particle,
    >();
    let mut var_r6_2: *mut *mut HSD_Particle = ::core::ptr::null_mut::<
        *mut HSD_Particle,
    >();
    let mut var_r7_2: *mut *mut HSD_Particle = ::core::ptr::null_mut::<
        *mut HSD_Particle,
    >();
    let mut i: s32 = 0;
    let mut temp_r3: u32 = 0;
    let mut temp_r3_2: u32 = 0;
    let mut temp_r4: s32 = 0;
    let mut var_r0: s32 = 0;
    let mut var_r0_2: s32 = 0;
    let mut var_r6: s32 = 0;
    let mut temp_r9: *mut u8 = ::core::ptr::null_mut::<u8>();
    temp_r9 = (&raw mut HSD_PSDisp_8040C360 as *mut u8).offset(arg0 as isize)
        as *mut u8;
    new_var = (&raw mut hsd_804D0908 as *mut *mut HSD_Particle).offset(arg0 as isize)
        as *mut *mut HSD_Particle;
    temp_r29 = new_var;
    var_r28 = *temp_r29;
    if *temp_r9 as i32 == arg1 as i32 {
        *arg2 = var_r28;
        *arg3 = particle_list[arg0 as usize];
        return var_r28;
    }
    *temp_r9 = arg1;
    if var_r28.is_null() {
        particle_list[arg0 as usize] = ::core::ptr::null_mut::<HSD_Particle>();
        *arg2 = ::core::ptr::null_mut::<HSD_Particle>();
        *arg3 = ::core::ptr::null_mut::<HSD_Particle>();
        return ::core::ptr::null_mut::<HSD_Particle>();
    }
    memset(
        &raw mut buckets as *mut psdisp_ParticleSortBucket as *mut ::core::ffi::c_void,
        0 as i32,
        ::core::mem::size_of::<[psdisp_ParticleSortBucket; 16]>() as usize,
    );
    temp_r3 = (*var_r28).kind;
    if temp_r3 & 8 as u32 != 0 {
        var_r0 = 0 as i32 as s32;
    } else {
        var_r0 = 1 as i32 as s32;
    }
    temp_r4 = (temp_r3 >> 0x19 as i32 & 7 as u32)
        .wrapping_add((var_r0 as i32 * 8 as i32) as u32)
        as s32;
    buckets[temp_r4 as usize].head = var_r28;
    var_r6 = temp_r4;
    var_r7 = (*var_r28).next;
    while !var_r7.is_null() {
        if ((*var_r28).kind ^ (*var_r7).kind) & 0xe000008 as i32 as u32
            != 0
        {
            buckets[var_r6 as usize].tail = var_r28;
            temp_r3_2 = (*var_r7).kind;
            if temp_r3_2 & 8 as u32 != 0 {
                var_r0_2 = 0 as i32 as s32;
            } else {
                var_r0_2 = 1 as i32 as s32;
            }
            var_r6 = (temp_r3_2 >> 0x19 as i32 & 7 as u32)
                .wrapping_add(
                    (var_r0_2 as i32 * 8 as i32) as u32,
                ) as s32;
            if buckets[var_r6 as usize].head.is_null() {
                buckets[var_r6 as usize].head = var_r7;
            } else {
                (*buckets[var_r6 as usize].tail).next = var_r7;
            }
        }
        var_r28 = var_r7;
        var_r7 = (*var_r7).next;
    }
    buckets[var_r6 as usize].tail = var_r28;
    var_r6_2 = ::core::ptr::null_mut::<*mut HSD_Particle>();
    var_r4 = ::core::ptr::null_mut::<HSD_Particle>();
    var_r7_2 = ::core::ptr::null_mut::<*mut HSD_Particle>();
    var_r5 = ::core::ptr::null_mut::<HSD_Particle>();
    i = 0 as i32 as s32;
    while i < 8 as i32 {
        if !buckets[i as usize].head.is_null() {
            if var_r4.is_null() {
                var_r4 = buckets[i as usize].head;
            } else {
                *var_r6_2 = buckets[i as usize].head;
            }
            var_r6_2 = &raw mut (*(*(&raw mut buckets as *mut psdisp_ParticleSortBucket)
                .offset(i as isize))
                .tail)
                .next;
        }
        i += 1;
    }
    i = 8 as i32 as s32;
    while i < 16 as i32 {
        if !buckets[i as usize].head.is_null() {
            if var_r5.is_null() {
                var_r5 = buckets[i as usize].head;
            } else {
                *var_r7_2 = buckets[i as usize].head;
            }
            var_r7_2 = &raw mut (*(*(&raw mut buckets as *mut psdisp_ParticleSortBucket)
                .offset(i as isize))
                .tail)
                .next;
        }
        i += 1;
    }
    var_r3 = ::core::ptr::null_mut::<HSD_Particle>();
    if !var_r6_2.is_null() {
        var_r3 = var_r4;
        *var_r6_2 = var_r5;
    }
    if !var_r7_2.is_null() {
        if var_r3.is_null() {
            var_r3 = var_r5;
        }
        *var_r7_2 = ::core::ptr::null_mut::<HSD_Particle>();
    }
    *temp_r29 = var_r3;
    particle_list[arg0 as usize] = var_r5;
    *arg2 = var_r3;
    *arg3 = var_r5;
    return var_r3;
}

pub unsafe fn psDispSubPoint(mut pp: *mut HSD_Particle) -> *mut HSD_Particle {
    melee_core::ct!("psDispSubPoint");
    let mut buf: [Vec3; 16] = [Vec3 { x: 0., y: 0., z: 0. }; 16];
    let mut p: *mut Vec3 = ::core::ptr::null_mut::<Vec3>();
    let mut last: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    let mut q: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    let mut count: s32 = 0;
    let mut i: s32 = 0;
    let mut w: u8 = 0;
    psSetCurrentMtx(GX_PNMTX0);
    w = (if (*pp).size as f64 > 42.5f64 {
        255.0f32
    } else {
        6.0f32 * (*pp).size
    }) as i32 as u8;
    if prevPointSize != w as s32 {
        prevPointSize = w as s32;
        GXSetPointSize(w, GX_TO_ONE);
    }
    last = pp;
    p = &raw mut buf as *mut Vec3;
    (*p).x = (*pp).pos.x;
    (*p).y = (*pp).pos.y;
    (*p).z = (*pp).pos.z;
    p = p.offset(1);
    count = 1 as i32 as s32;
    q = (*pp).next;
    while !q.is_null() {
        if !((*q).size == (*pp).size && (*q).appsrt.is_null()
            && ((*q).kind as u32 ^ (*pp).kind as u32)
                & 0xc0100400 as u32 == 0
            && (*q).primColCount as i32 == 0 as i32
            && (*q).primCol.r as i32
                == (*pp).primCol.r as i32
            && (*q).primCol.g as i32
                == (*pp).primCol.g as i32
            && (*q).primCol.b as i32
                == (*pp).primCol.b as i32
            && (*q).primCol.a as i32
                == (*pp).primCol.a as i32
            && (*q).kind & DispPoint as i32 as u32 == 0
            && ((*pp).kind & DispLighting as i32 as u32 == 0
                || (*q).matColCount as i32 == 0 as i32
                    && (*q).ambColCount as i32 == 0 as i32
                    && (*q).matRGB as i32
                        == (*pp).matRGB as i32
                    && (*q).matA as i32
                        == (*pp).matA as i32
                    && (*q).ambRGB as i32
                        == (*pp).ambRGB as i32
                    && (*q).ambA as i32
                        == (*pp).ambA as i32))
        {
            break;
        }
        count += 1;
        (*p).x = (*q).pos.x;
        (*p).y = (*q).pos.y;
        (*p).z = (*q).pos.z;
        p = p.offset(1);
        if count == 16 as i32 {
            p = &raw mut buf as *mut Vec3;
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                setVtxDesc(0 as s32);
                GXBegin(GX_POINTS, GX_VTXFMT0, 16 as u16);
            } else {
                setVtxDesc(1 as s32);
                GXBegin(GX_POINTS, GX_VTXFMT1, 16 as u16);
            }
            i = count;
            while i != 0 as i32 {
                GXPosition3f32((*p).x, (*p).y, (*p).z);
                p = p.offset(1);
                if (*pp).kind & DispTexture as i32 as u32 != 0 {
                    GXTexCoord1x8(1 as i32 as u8);
                }
                i -= 1;
            }
            p = &raw mut buf as *mut Vec3;
            count = 0 as i32 as s32;
        }
        last = q;
        q = (*last).next;
    }
    if count != 0 as i32 {
        p = &raw mut buf as *mut Vec3;
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            setVtxDesc(0 as s32);
            GXBegin(GX_POINTS, GX_VTXFMT0, count as u16);
        } else {
            setVtxDesc(1 as s32);
            GXBegin(GX_POINTS, GX_VTXFMT1, count as u16);
        }
        i = count;
        while i != 0 as i32 {
            GXPosition3f32((*p).x, (*p).y, (*p).z);
            p = p.offset(1);
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                GXTexCoord1x8(1 as i32 as u8);
            }
            i -= 1;
        }
        let _ = !q.is_null();
    }
    return last;
}

pub unsafe fn psDispSubPointTrail(
    mut pp: *mut HSD_Particle,
) -> *mut HSD_Particle {
    melee_core::ct!("psDispSubPointTrail");
    let mut vbuf: [Vec3; 32] = [Vec3 { x: 0., y: 0., z: 0. }; 32];
    let mut cbuf: [GXColor; 32] = [_GXColor { r: 0, g: 0, b: 0, a: 0 }; 32];
    let mut p: *mut Vec3 = ::core::ptr::null_mut::<Vec3>();
    let mut c: *mut GXColor = ::core::ptr::null_mut::<GXColor>();
    let mut last: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    let mut q: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    let mut count: s32 = 0;
    let mut i: s32 = 0;
    let mut w: u8 = 0;
    psSetCurrentMtx(GX_PNMTX0);
    w = (if (*pp).size as f64 > 42.5f64 {
        255.0f32
    } else {
        6.0f32 * (*pp).size
    }) as i32 as u8;
    if prevLineWidth != w as s32 {
        prevLineWidth = w as s32;
        GXSetLineWidth(w, GX_TO_ONE);
    }
    p = &raw mut vbuf as *mut Vec3;
    c = &raw mut cbuf as *mut GXColor;
    let fresh12 = p;
    p = p.offset(1);
    let mut dst: *mut Vec3 = fresh12;
    (*dst).x = (*pp).pos.x;
    (*dst).y = (*pp).pos.y;
    (*dst).z = (*pp).pos.z;
    if (*pp).kind & Tornado as i32 as u32 != 0 {
        let mut x: f32 = 0.;
        let mut y: f32 = 0.;
        let mut z: f32 = 0.;
        calcTornadoLastPos(pp, &raw mut x, &raw mut y, &raw mut z);
        (*p).x = x;
        (*p).y = y;
        (*p).z = z;
        p = p.offset(1);
    } else {
        (*p).x = (*pp).pos.x - (*pp).vel.x;
        (*p).y = (*pp).pos.y - (*pp).vel.y;
        (*p).z = (*pp).pos.z - (*pp).vel.z;
        p = p.offset(1);
    }
    getClrTrail(pp, c);
    *c.offset(1 as i32 as isize) = *c
        .offset(0 as i32 as isize);
    count = 1 as i32 as s32;
    (*c.offset(1 as i32 as isize)).a = ((*c
        .offset(1 as i32 as isize))
        .a as f32 * (*pp).trail) as i32 as u8;
    c = c.offset(2 as i32 as isize);
    last = pp;
    q = (*pp).next;
    while !q.is_null() {
        if !((*q).size == (*pp).size && (*q).appsrt.is_null()
            && ((*q).kind as u32 ^ (*pp).kind as u32)
                & 0xc0100400 as u32 == 0
            && (*q).kind & DispPoint as i32 as u32 == 0)
        {
            break;
        }
        let fresh13 = p;
        p = p.offset(1);
        let mut dst_0: *mut Vec3 = fresh13;
        (*dst_0).x = (*q).pos.x;
        (*dst_0).y = (*q).pos.y;
        (*dst_0).z = (*q).pos.z;
        if (*q).kind & Tornado as i32 as u32 != 0 {
            let mut x_0: f32 = 0.;
            let mut y_0: f32 = 0.;
            let mut z_0: f32 = 0.;
            calcTornadoLastPos(q, &raw mut x_0, &raw mut y_0, &raw mut z_0);
            (*p).x = x_0;
            (*p).y = y_0;
            (*p).z = z_0;
            p = p.offset(1);
        } else {
            (*p).x = (*q).pos.x - (*q).vel.x;
            (*p).y = (*q).pos.y - (*q).vel.y;
            (*p).z = (*q).pos.z - (*q).vel.z;
            p = p.offset(1);
        }
        getClrTrail(q, c);
        *c.offset(1 as i32 as isize) = *c
            .offset(0 as i32 as isize);
        count += 1;
        (*c.offset(1 as i32 as isize)).a = ((*c
            .offset(1 as i32 as isize))
            .a as f32 * (*q).trail) as i32 as u8;
        c = c.offset(2 as i32 as isize);
        if count == 16 as i32 {
            p = &raw mut vbuf as *mut Vec3;
            c = &raw mut cbuf as *mut GXColor;
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                setVtxDesc(2 as s32);
                GXBegin(GX_LINES, GX_VTXFMT2, 0x20 as u16);
            } else {
                setVtxDesc(3 as s32);
                GXBegin(GX_LINES, GX_VTXFMT3, 0x20 as u16);
            }
            i = count;
            while i != 0 as i32 {
                GXPosition3f32(
                    (*p.offset(1 as i32 as isize)).x,
                    (*p.offset(1 as i32 as isize)).y,
                    (*p.offset(1 as i32 as isize)).z,
                );
                GXColor4u8(
                    (*c.offset(1 as i32 as isize)).r,
                    (*c.offset(1 as i32 as isize)).g,
                    (*c.offset(1 as i32 as isize)).b,
                    (*c.offset(1 as i32 as isize)).a,
                );
                if (*pp).kind & DispTexture as i32 as u32 != 0 {
                    GXTexCoord1x8(0 as i32 as u8);
                }
                GXPosition3f32(
                    (*p.offset(0 as i32 as isize)).x,
                    (*p.offset(0 as i32 as isize)).y,
                    (*p.offset(0 as i32 as isize)).z,
                );
                GXColor4u8(
                    (*c.offset(0 as i32 as isize)).r,
                    (*c.offset(0 as i32 as isize)).g,
                    (*c.offset(0 as i32 as isize)).b,
                    (*c.offset(0 as i32 as isize)).a,
                );
                if (*pp).kind & DispTexture as i32 as u32 != 0 {
                    GXTexCoord1x8(1 as i32 as u8);
                }
                p = p.offset(2 as i32 as isize);
                c = c.offset(2 as i32 as isize);
                i -= 1;
            }
            p = &raw mut vbuf as *mut Vec3;
            c = &raw mut cbuf as *mut GXColor;
            count = 0 as i32 as s32;
        }
        last = q;
        q = (*q).next;
    }
    if count != 0 as i32 {
        let mut draw_colors: *mut GXColor = ::core::ptr::null_mut::<GXColor>();
        p = &raw mut vbuf as *mut Vec3;
        draw_colors = &raw mut cbuf as *mut GXColor;
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            setVtxDesc(2 as s32);
            GXBegin(
                GX_LINES,
                GX_VTXFMT2,
                (count as i32 * 2 as i32) as u16,
            );
        } else {
            setVtxDesc(3 as s32);
            GXBegin(
                GX_LINES,
                GX_VTXFMT3,
                (count as i32 * 2 as i32) as u16,
            );
        }
        i = count;
        while i != 0 as i32 {
            GXPosition3f32(
                (*p.offset(1 as i32 as isize)).x,
                (*p.offset(1 as i32 as isize)).y,
                (*p.offset(1 as i32 as isize)).z,
            );
            GXColor4u8(
                (*draw_colors.offset(1 as i32 as isize)).r,
                (*draw_colors.offset(1 as i32 as isize)).g,
                (*draw_colors.offset(1 as i32 as isize)).b,
                (*draw_colors.offset(1 as i32 as isize)).a,
            );
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                GXTexCoord1x8(0 as i32 as u8);
            }
            GXPosition3f32(
                (*p.offset(0 as i32 as isize)).x,
                (*p.offset(0 as i32 as isize)).y,
                (*p.offset(0 as i32 as isize)).z,
            );
            GXColor4u8(
                (*draw_colors.offset(0 as i32 as isize)).r,
                (*draw_colors.offset(0 as i32 as isize)).g,
                (*draw_colors.offset(0 as i32 as isize)).b,
                (*draw_colors.offset(0 as i32 as isize)).a,
            );
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                GXTexCoord1x8(1 as i32 as u8);
            }
            p = p.offset(2 as i32 as isize);
            draw_colors = draw_colors.offset(2 as i32 as isize);
            i -= 1;
        }
    }
    return last;
}

pub unsafe fn setBlendMode(mut blend_mode: i32) {
    melee_core::ct!("setBlendMode");
    if HSD_PSDisp_804D792C != blend_mode {
        HSD_PSDisp_804D792C = blend_mode as s32;
        match blend_mode {
            0 => {
                GXSetBlendMode(
                    GX_BM_BLEND,
                    GX_BL_SRCALPHA,
                    GX_BL_INVSRCALPHA,
                    GX_LO_CLEAR,
                );
            }
            1 => {
                GXSetBlendMode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_ONE, GX_LO_CLEAR);
            }
            _ => {
                OSReport(
                    b"Particle:setBlendMode:Unknown mode\n\0" as *const u8
                        as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
            }
        }
    }
}

pub unsafe fn psSetCurrentMtx(mut idx: GXPosNrmMtx) {
    melee_core::ct!("psSetCurrentMtx");
    if HSD_PSDisp_804D7948[0 as i32 as usize] != idx as i32
    {
        HSD_PSDisp_804D7948[0 as i32 as usize] = idx as s32;
        GXSetCurrentMtx(idx as u32);
    }
}

pub unsafe fn psDispSubMakePolygon(
    mut pp: *mut HSD_Particle,
    mut texform: *mut u8,
    mut x: f32,
    mut y: f32,
    mut z: f32,
    mut x0: f32,
    mut y0: f32,
    mut z0: f32,
    mut x1: f32,
    mut y1: f32,
    mut z1: f32,
    mut color: *mut GXColor,
    mut prev_x: *mut f32,
    mut prev_y: *mut f32,
    mut prev_z: *mut f32,
) {
    melee_core::ct!("psDispSubMakePolygon");
    let mut right: Vec2 = Vec2 { x: 0., y: 0. };
    let mut right_z: f32 = 0.;
    let mut up: Vec2 = Vec2 { x: 0., y: 0. };
    let mut up_z: f32 = 0.;
    let mut it: *mut u8 = texform;
    let mut cx: f32 = 0.;
    right.x = x0;
    right.y = y0;
    right_z = z0;
    up.x = x1;
    up.y = y1;
    up_z = z1;
    psSetCurrentMtx(GX_PNMTX0);
    if (*pp).kind & Trail as i32 as u32 != 0 {
        if (*pp).kind & Tornado as i32 as u32 != 0 {
            calcTornadoLastPos(pp, prev_x, prev_y, prev_z);
        } else {
            *prev_x = x - (*pp).vel.x;
            *prev_y = y - (*pp).vel.y;
            *prev_z = z - (*pp).vel.z;
        }
        getClrTrail(pp, color);
        if it.is_null() {
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                setVtxDesc(2 as s32);
                GXBegin(GX_QUADS, GX_VTXFMT2, 4 as u16);
            } else {
                setVtxDesc(3 as s32);
                GXBegin(GX_QUADS, GX_VTXFMT3, 4 as u16);
            }
            GXPosition3f32(*prev_x - right.x, *prev_y - right.y, *prev_z - right_z);
            let mut a: u8 = (*color).a;
            let mut b: u8 = (*color).b;
            let mut g: u8 = (*color).g;
            let mut r: u16 = (*color).r as u16;
            GXColor4u8(
                r as i32 as u8,
                g,
                b,
                (a as f32 * (*pp).trail) as i32 as u8,
            );
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                let mut tex_base: u8 = ((*pp).kind >> 16 as i32
                    & 0xc as u32) as i32 as u8;
                GXTexCoord1x8(tex_base);
            }
            GXPosition3f32(x - up.x, y - up.y, z - up_z);
            let mut a_0: u8 = (*color).a;
            let mut b_0: u8 = (*color).b;
            let mut g_0: u8 = (*color).g;
            let mut r_0: u16 = (*color).r as u16;
            GXColor4u8(r_0 as i32 as u8, g_0, b_0, a_0);
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                GXTexCoord1x8(
                    ((*pp).kind >> 16 as i32 & 0xc as u32)
                        .wrapping_add(1 as u32) as i32 as u8,
                );
            }
            GXPosition3f32(x + right.x, y + right.y, z + right_z);
            let mut a_1: u8 = (*color).a;
            let mut b_1: u8 = (*color).b;
            let mut g_1: u8 = (*color).g;
            let mut r_1: u16 = (*color).r as u16;
            GXColor4u8(r_1 as i32 as u8, g_1, b_1, a_1);
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                GXTexCoord1x8(
                    ((*pp).kind >> 16 as i32 & 0xc as u32)
                        .wrapping_add(2 as u32) as i32 as u8,
                );
            }
            GXPosition3f32(*prev_x + up.x, *prev_y + up.y, *prev_z + up_z);
            let mut a_2: u8 = (*color).a;
            let mut alpha: f32 = a_2 as f32 * (*pp).trail as f32;
            let mut b_2: u8 = (*color).b;
            let mut g_2: u8 = (*color).g;
            let mut r_2: u16 = (*color).r as u16;
            GXColor4u8(r_2 as i32 as u8, g_2, b_2, alpha as i32 as u8);
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                GXTexCoord1x8(
                    ((*pp).kind >> 16 as i32 & 0xc as u32)
                        .wrapping_add(3 as u32) as i32 as u8,
                );
            }
        } else {
            let mut trail_alpha: f32 = 255.0f32 * (1.0f32 - (*pp).trail as f32);
            let mut up_len: f32 = sqrtf(
                fmadds(
                    up_z as f32,
                    up_z as f32,
                    fmadds(
                        up.x as f32,
                        up.x as f32,
                        up.y as f32 * up.y as f32,
                    ),
                ),
            ) as f32;
            if up_len != 0.0f32 {
                let mut dz: f32 = z - *prev_z;
                let mut dx: f32 = x - *prev_x;
                let mut dy: f32 = y - *prev_y;
                let mut xl: f32 = dx * dx;
                let mut yl: f32 = dy * dy;
                let mut zl: f32 = dz * dz;
                let mut segment_len: f32 = sqrtf(
                    zl as f32
                        + (xl as f32 + yl as f32),
                ) as f32;
                let mut ratio: f32 = segment_len / up_len;
                let mut primitive_count: u32 = *(it as *mut u32);
                it = it.offset(::core::mem::size_of::<u32>() as usize as isize);
                up.x *= ratio;
                up.y *= ratio;
                up_z *= ratio;
                while primitive_count != 0 as u32 {
                    let mut primitive: GXPrimitive = *it
                        .offset(0 as i32 as isize) as GXPrimitive;
                    let mut count: u8 = *it.offset(1 as i32 as isize);
                    let mut i: s32 = 0;
                    it = it.offset(4 as i32 as isize);
                    if (*pp).kind & DispTexture as i32 as u32 != 0 {
                        setVtxDesc(5 as s32);
                        GXBegin(primitive, GX_VTXFMT5, count as u16);
                    } else {
                        setVtxDesc(3 as s32);
                        GXBegin(primitive, GX_VTXFMT3, count as u16);
                    }
                    i = count as s32;
                    while i > 0 as i32 {
                        let mut s: f32 = *(it.offset(0 as i32 as isize)
                            as *mut u8 as *mut f32);
                        let mut sx: f32 = 2.0f32 * (s - 0.5f32);
                        let mut t: f32 = 0.;
                        let mut tx: f32 = 0.;
                        let mut converted_alpha: s32 = 0;
                        let mut alpha_0: s32 = 0;
                        if (*pp).kind & TexFlipS as i32 as u32 != 0 {
                            s = 1.0f32 - s;
                        }
                        t = *(it.offset(4 as i32 as isize) as *mut u8
                            as *mut f32);
                        it = it.offset(8 as i32 as isize);
                        alpha_0 = fmadds(
                            -(t as f32),
                            trail_alpha as f32,
                            255.0f32,
                        ) as s32;
                        converted_alpha = fmadds(
                            -(t as f32),
                            trail_alpha as f32,
                            255.0f32,
                        ) as s32;
                        if converted_alpha < 0 as i32 {
                            alpha_0 = 0 as i32 as s32;
                        }
                        if alpha_0 > 0xff as i32 {
                            alpha_0 = 0xff as i32 as s32;
                        }
                        tx = (2.0f32 * (t as f32 - 0.5f32)) as f32;
                        if (*pp).kind & TexFlipT as i32 as u32 != 0 {
                            t = 1.0f32 - t;
                        }
                        GXPosition3f32(
                            fmadds(
                                up.x as f32,
                                tx as f32,
                                fmadds(
                                    right.x as f32,
                                    sx as f32,
                                    x as f32,
                                ),
                            ) as f32,
                            fmadds(
                                up.y as f32,
                                tx as f32,
                                fmadds(
                                    right.y as f32,
                                    sx as f32,
                                    y as f32,
                                ),
                            ) as f32,
                            fmadds(
                                up_z as f32,
                                tx as f32,
                                fmadds(
                                    right_z as f32,
                                    sx as f32,
                                    z as f32,
                                ),
                            ) as f32,
                        );
                        GXColor4u8((*color).r, (*color).g, (*color).b, alpha_0 as i32 as u8);
                        if (*pp).kind & DispTexture as i32 as u32 != 0 {
                            GXTexCoord2f32(s, t);
                        }
                        i -= 1;
                    }
                    primitive_count = primitive_count.wrapping_sub(1);
                }
            }
        }
    } else if it.is_null() {
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            setVtxDesc(0 as s32);
            GXBegin(GX_QUADS, GX_VTXFMT0, 4 as u16);
        } else {
            setVtxDesc(1 as s32);
            GXBegin(GX_QUADS, GX_VTXFMT1, 4 as u16);
        }
        cx = x - right.x;
        GXPosition3f32(cx, y - right.y, z - right_z);
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            GXTexCoord1x8(
                ((*pp).kind >> 16 as i32 & 0xc as u32) as i32 as u8,
            );
        }
        cx = x - up.x;
        GXPosition3f32(cx, y - up.y, z - up_z);
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            GXTexCoord1x8(
                ((*pp).kind >> 16 as i32 & 0xc as u32)
                    .wrapping_add(1 as u32) as i32 as u8,
            );
        }
        cx = x + right.x;
        GXPosition3f32(cx, y + right.y, z + right_z);
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            GXTexCoord1x8(
                ((*pp).kind >> 16 as i32 & 0xc as u32)
                    .wrapping_add(2 as u32) as i32 as u8,
            );
        }
        GXPosition3f32(x + up.x, y + up.y, z + up_z);
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            GXTexCoord1x8(
                ((*pp).kind >> 16 as i32 & 0xc as u32)
                    .wrapping_add(3 as u32) as i32 as u8,
            );
        }
    } else {
        let mut primitive_count_0: u32 = *(it as *mut u32);
        it = it.offset(::core::mem::size_of::<u32>() as usize as isize);
        while primitive_count_0 != 0 as u32 {
            let mut primitive_0: GXPrimitive = *it
                .offset(0 as i32 as isize) as GXPrimitive;
            let mut count_0: u8 = *it.offset(1 as i32 as isize);
            let mut i_0: s32 = 0;
            it = it.offset(4 as i32 as isize);
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                setVtxDesc(4 as s32);
                GXBegin(primitive_0, GX_VTXFMT4, count_0 as u16);
            } else {
                setVtxDesc(1 as s32);
                GXBegin(primitive_0, GX_VTXFMT1, count_0 as u16);
            }
            i_0 = count_0 as s32;
            while i_0 > 0 as i32 {
                let mut s_0: f32 = *(it.offset(0 as i32 as isize)
                    as *mut u8 as *mut f32);
                let mut sx_0: f32 = 2.0f32 * (s_0 - 0.5f32);
                let mut t_0: f32 = 0.;
                let mut tx_0: f32 = 0.;
                if (*pp).kind & TexFlipS as i32 as u32 != 0 {
                    s_0 = 1.0f32 - s_0;
                }
                t_0 = *(it.offset(4 as i32 as isize) as *mut u8
                    as *mut f32);
                it = it.offset(8 as i32 as isize);
                tx_0 = (2.0f32 * (t_0 as f32 - 0.5f32)) as f32;
                if (*pp).kind & TexFlipT as i32 as u32 != 0 {
                    t_0 = 1.0f32 - t_0;
                }
                GXPosition3f32(
                    fmadds(
                        up.x as f32,
                        tx_0 as f32,
                        fmadds(
                            right.x as f32,
                            sx_0 as f32,
                            x as f32,
                        ),
                    ) as f32,
                    fmadds(
                        up.y as f32,
                        tx_0 as f32,
                        fmadds(
                            right.y as f32,
                            sx_0 as f32,
                            y as f32,
                        ),
                    ) as f32,
                    fmadds(
                        up_z as f32,
                        tx_0 as f32,
                        fmadds(
                            right_z as f32,
                            sx_0 as f32,
                            z as f32,
                        ),
                    ) as f32,
                );
                if (*pp).kind & DispTexture as i32 as u32 != 0 {
                    GXTexCoord2f32(s_0, t_0);
                }
                i_0 -= 1;
            }
            primitive_count_0 = primitive_count_0.wrapping_sub(1);
        }
    };
}

pub unsafe fn psMaskAbsF32(mut value: *mut f32) {
    melee_core::ct!("psMaskAbsF32");
    let ref mut fresh2 = *(value as *mut s32);
    *fresh2 &= 0x7fffffff as i32;
}

pub unsafe fn psMaskAbsLtF32(mut value: f32, mut limit: f32) -> i32 {
    melee_core::ct!("psMaskAbsLtF32");
    psMaskAbsF32(&raw mut value);
    return (value < limit) as i32;
}

pub unsafe fn psMaskAbsGtF32(mut value: f32, mut limit: f64) -> i32 {
    melee_core::ct!("psMaskAbsGtF32");
    psMaskAbsF32(&raw mut value);
    return (value as f64 > limit) as i32;
}

pub unsafe fn psAbsLtF32(mut value: f32, mut limit: f32) -> i32 {
    melee_core::ct!("psAbsLtF32");
    let ref mut fresh1 = *(&raw mut value as *mut s32);
    *fresh1 &= 0x7fffffff as i32;
    return (value < limit) as i32;
}

pub unsafe fn psAbsGtF32(mut value: f32, mut limit: f64) -> i32 {
    melee_core::ct!("psAbsGtF32");
    let ref mut fresh0 = *(&raw mut value as *mut s32);
    *fresh0 &= 0x7fffffff as i32;
    return (value as f64 > limit) as i32;
}

pub unsafe fn psDispSubAbsLtF32(mut value: f32, mut limit: f32) -> i32 {
    melee_core::ct!("psDispSubAbsLtF32");
    return psAbsLtF32(value, limit);
}

pub unsafe fn psDispSubAbsGtF32(mut value: f32, mut limit: f64) -> i32 {
    melee_core::ct!("psDispSubAbsGtF32");
    return psAbsGtF32(value, limit);
}

pub unsafe fn psDispSub(mut pp: *mut HSD_Particle, mut texform: *mut u8) {
    melee_core::ct!("psDispSub");
    let mut right_y: f32 = 0.;
    let mut right_x: f32 = 0.;
    let mut right_z: f32 = 0.;
    let mut up_x: f32 = 0.;
    let mut up_y: f32 = 0.;
    let mut up_z: f32 = 0.;
    let mut angle: f32 = 0.;
    let mut x: f32 = 0.;
    let mut y: f32 = 0.;
    let mut z: f32 = 0.;
    x = (*pp).pos.x;
    y = (*pp).pos.y;
    z = (*pp).pos.z;
    if !texform.is_null() {
        right_x = (rvmtx[0 as i32
            as usize][0 as i32 as usize] * (*pp).size) as f32;
        up_x = (-rvmtx[0 as i32
            as usize][1 as i32 as usize] * (*pp).size) as f32;
        right_y = (rvmtx[1 as i32
            as usize][0 as i32 as usize] * (*pp).size) as f32;
        up_y = (-rvmtx[1 as i32
            as usize][1 as i32 as usize] * (*pp).size) as f32;
        right_z = (rvmtx[2 as i32
            as usize][0 as i32 as usize] * (*pp).size) as f32;
        up_z = (-rvmtx[2 as i32
            as usize][1 as i32 as usize] * (*pp).size) as f32;
    } else {
        right_x = (HSD_PSDisp_804D7914 as f32 * (*pp).size) as f32;
        up_x = (HSD_PSDisp_804D7918 as f32 * (*pp).size) as f32;
        right_y = (HSD_PSDisp_804D791C as f32 * (*pp).size) as f32;
        up_y = (HSD_PSDisp_804D7920 as f32 * (*pp).size) as f32;
        right_z = (HSD_PSDisp_804D7924 as f32 * (*pp).size) as f32;
        up_z = (HSD_PSDisp_804D7928 as f32 * (*pp).size) as f32;
    }
    if (*pp).kind & Trail as i32 as u32 != 0
        || (*pp).kind & DirVec as i32 as u32 != 0
    {
        let mut x_0: f32 = 0.;
        let mut y_0: f32 = 0.;
        if 0.0f32 == prj[0 as i32 as usize] {
            let mut prev_x: f32 = 0.;
            let mut prev_y: f32 = 0.;
            let mut prev_z: f32 = 0.;
            let mut w0: f32 = 0.;
            let mut w0inv: f32 = 0.;
            let mut w1: f32 = 0.;
            let mut w1inv: f32 = 0.;
            let mut pv13: f32 = 0.;
            let mut pv03: f32 = 0.;
            if (*pp).kind & Tornado as i32 as u32 != 0 {
                calcTornadoLastPos(
                    pp,
                    &raw mut prev_x,
                    &raw mut prev_y,
                    &raw mut prev_z,
                );
            } else {
                prev_x = (*pp).pos.x - (*pp).vel.x;
                prev_y = (*pp).pos.y - (*pp).vel.y;
                prev_z = (*pp).pos.z - (*pp).vel.z;
            }
            w0 = (vmtx[2 as i32
                as usize][3 as i32 as usize]
                + fmadds(
                    vmtx[2 as i32
                        as usize][2 as i32 as usize],
                    (*pp).pos.z as f32,
                    fmadds(
                        vmtx[2 as i32
                            as usize][0 as i32 as usize],
                        (*pp).pos.x as f32,
                        vmtx[2 as i32
                            as usize][1 as i32 as usize]
                            * (*pp).pos.y as f32,
                    ),
                )) as f32;
            if 0.0f32 == w0 {
                return;
            }
            w0inv = -1.0f32 / w0;
            w1 = (vmtx[2 as i32
                as usize][3 as i32 as usize]
                + fmadds(
                    vmtx[2 as i32
                        as usize][2 as i32 as usize],
                    prev_z as f32,
                    fmadds(
                        vmtx[2 as i32
                            as usize][0 as i32 as usize],
                        prev_x as f32,
                        vmtx[2 as i32
                            as usize][1 as i32 as usize]
                            * prev_y as f32,
                    ),
                )) as f32;
            if 0.0f32 == w1 {
                return;
            }
            w1inv = -1.0f32 / w1;
            pv03 = pvmtx[0 as i32
                as usize][3 as i32 as usize];
            pv13 = pvmtx[1 as i32
                as usize][3 as i32 as usize];
            x_0 = fmadds(
                w0inv as f32,
                pv03 as f32
                    + fmadds(
                        pvmtx[0 as i32
                            as usize][2 as i32 as usize],
                        (*pp).pos.z as f32,
                        fmadds(
                            pvmtx[0 as i32
                                as usize][0 as i32 as usize],
                            (*pp).pos.x as f32,
                            pvmtx[0 as i32
                                as usize][1 as i32 as usize]
                                * (*pp).pos.y as f32,
                        ),
                    ),
                -(w1inv as f32
                    * (pv03 as f32
                        + fmadds(
                            pvmtx[0 as i32
                                as usize][2 as i32 as usize],
                            prev_z as f32,
                            fmadds(
                                pvmtx[0 as i32
                                    as usize][0 as i32 as usize],
                                prev_x as f32,
                                pvmtx[0 as i32
                                    as usize][1 as i32 as usize]
                                    * prev_y as f32,
                            ),
                        ))),
            ) as f32;
            y_0 = fmadds(
                w0inv as f32,
                pv13 as f32
                    + fmadds(
                        pvmtx[1 as i32
                            as usize][2 as i32 as usize],
                        (*pp).pos.z as f32,
                        fmadds(
                            pvmtx[1 as i32
                                as usize][0 as i32 as usize],
                            (*pp).pos.x as f32,
                            pvmtx[1 as i32
                                as usize][1 as i32 as usize]
                                * (*pp).pos.y as f32,
                        ),
                    ),
                -(w1inv as f32
                    * (pv13 as f32
                        + fmadds(
                            pvmtx[1 as i32
                                as usize][2 as i32 as usize],
                            prev_z as f32,
                            fmadds(
                                pvmtx[1 as i32
                                    as usize][0 as i32 as usize],
                                prev_x as f32,
                                pvmtx[1 as i32
                                    as usize][1 as i32 as usize]
                                    * prev_y as f32,
                            ),
                        ))),
            ) as f32;
        } else if (*pp).kind & Tornado as i32 as u32 != 0 {
            let mut prev_x_0: f32 = 0.;
            let mut prev_y_0: f32 = 0.;
            let mut prev_z_0: f32 = 0.;
            let mut dx: f32 = 0.;
            let mut dy: f32 = 0.;
            let mut dz: f32 = 0.;
            calcTornadoLastPos(
                pp,
                &raw mut prev_x_0,
                &raw mut prev_y_0,
                &raw mut prev_z_0,
            );
            dx = (*pp).pos.x - prev_x_0;
            dy = (*pp).pos.y - prev_y_0;
            dz = (*pp).pos.z - prev_z_0;
            x_0 = fmadds(
                pvmtx[0 as i32
                    as usize][2 as i32 as usize],
                dz as f32,
                fmadds(
                    pvmtx[0 as i32
                        as usize][0 as i32 as usize],
                    dx as f32,
                    pvmtx[0 as i32
                        as usize][1 as i32 as usize]
                        * dy as f32,
                ),
            ) as f32;
            y_0 = fmadds(
                pvmtx[1 as i32
                    as usize][2 as i32 as usize],
                dz as f32,
                fmadds(
                    pvmtx[1 as i32
                        as usize][0 as i32 as usize],
                    dx as f32,
                    pvmtx[1 as i32
                        as usize][1 as i32 as usize]
                        * dy as f32,
                ),
            ) as f32;
        } else {
            x_0 = fmadds(
                pvmtx[0 as i32
                    as usize][2 as i32 as usize],
                (*pp).vel.z as f32,
                fmadds(
                    pvmtx[0 as i32
                        as usize][0 as i32 as usize],
                    (*pp).vel.x as f32,
                    pvmtx[0 as i32
                        as usize][1 as i32 as usize]
                        * (*pp).vel.y as f32,
                ),
            ) as f32;
            y_0 = fmadds(
                pvmtx[1 as i32
                    as usize][2 as i32 as usize],
                (*pp).vel.z as f32,
                fmadds(
                    pvmtx[1 as i32
                        as usize][0 as i32 as usize],
                    (*pp).vel.x as f32,
                    pvmtx[1 as i32
                        as usize][1 as i32 as usize]
                        * (*pp).vel.y as f32,
                ),
            ) as f32;
        }
        if psDispSubAbsLtF32(y_0, FLT_MIN) != 0 {
            angle = (if x_0 >= 0.0f32 { 1.5707964f32 } else { -1.5707964f32 }) as f32;
        } else {
            angle = atan2f(x_0 as f32, y_0 as f32)
                as f32;
        }
        if (*pp).kind & DirVec as i32 as u32 != 0 {
            angle += (*pp).rotate;
        }
    } else {
        angle = (*pp).rotate as f32;
    }
    if psDispSubAbsGtF32(angle, 0.01f64) != 0 {
        let mut mtx: Mtx = [[0.; 4]; 3];
        let mut axis: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
        let mut rx: f32 = right_x;
        let mut ry: f32 = right_y;
        let mut rz: f32 = right_z;
        let mut ux: f32 = up_x;
        let mut uz: f32 = up_z;
        let mut uy: f32 = up_y;
        let mut ax: f32 = 0.;
        let mut ay: f32 = 0.;
        let mut az: f32 = 0.;
        ax = fmadds(
            ry as f32,
            uz as f32,
            -(rz as f32 * uy as f32),
        ) as f32;
        let mut axis_y_product: f32 = rx * uz;
        ay = fmadds(
            rz as f32,
            ux as f32,
            -(axis_y_product as f32),
        ) as f32;
        az = fmadds(
            rx as f32,
            uy as f32,
            -(ry as f32 * ux as f32),
        ) as f32;
        axis.x = ax;
        axis.y = ay;
        axis.z = az;
        PSMTXRotAxisRad(&raw mut mtx as *mut [f32; 4], &raw mut axis, angle);
        right_x = fmadds(
            mtx[0 as i32 as usize][2 as i32 as usize],
            rz as f32,
            fmadds(
                mtx[0 as i32 as usize][0 as i32 as usize],
                rx as f32,
                mtx[0 as i32 as usize][1 as i32 as usize]
                    * ry as f32,
            ),
        ) as f32;
        right_y = fmadds(
            mtx[1 as i32 as usize][2 as i32 as usize],
            rz as f32,
            fmadds(
                mtx[1 as i32 as usize][0 as i32 as usize],
                rx as f32,
                mtx[1 as i32 as usize][1 as i32 as usize]
                    * ry as f32,
            ),
        ) as f32;
        right_z = fmadds(
            mtx[2 as i32 as usize][2 as i32 as usize],
            rz as f32,
            fmadds(
                mtx[2 as i32 as usize][0 as i32 as usize],
                rx as f32,
                mtx[2 as i32 as usize][1 as i32 as usize]
                    * ry as f32,
            ),
        ) as f32;
        up_x = fmadds(
            mtx[0 as i32 as usize][2 as i32 as usize],
            uz as f32,
            fmadds(
                mtx[0 as i32 as usize][0 as i32 as usize],
                ux as f32,
                mtx[0 as i32 as usize][1 as i32 as usize]
                    * uy as f32,
            ),
        ) as f32;
        up_y = fmadds(
            mtx[1 as i32 as usize][2 as i32 as usize],
            uz as f32,
            fmadds(
                mtx[1 as i32 as usize][0 as i32 as usize],
                ux as f32,
                mtx[1 as i32 as usize][1 as i32 as usize]
                    * uy as f32,
            ),
        ) as f32;
        up_z = fmadds(
            mtx[2 as i32 as usize][2 as i32 as usize],
            uz as f32,
            fmadds(
                mtx[2 as i32 as usize][0 as i32 as usize],
                ux as f32,
                mtx[2 as i32 as usize][1 as i32 as usize]
                    * uy as f32,
            ),
        ) as f32;
    }
    let mut color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut prev_x_1: f32 = 0.;
    let mut prev_y_1: f32 = 0.;
    let mut prev_z_1: f32 = 0.;
    psDispSubMakePolygon(
        pp,
        texform,
        x,
        y,
        z,
        right_x,
        right_y,
        right_z,
        up_x,
        up_y,
        up_z,
        &raw mut color,
        &raw mut prev_x_1,
        &raw mut prev_y_1,
        &raw mut prev_z_1,
    );
}

pub unsafe fn psScaleAppSRTAxes(
    mut pp: *mut HSD_Particle,
    mut mtx: *mut [f32; 4],
) {
    melee_core::ct!("psScaleAppSRTAxes");
    let ref mut fresh3 = (*mtx
        .offset(0 as i32 as isize))[0 as i32 as usize];
    *fresh3 *= (*pp).size;
    let ref mut fresh4 = (*mtx
        .offset(1 as i32 as isize))[0 as i32 as usize];
    *fresh4 *= (*pp).size;
    let ref mut fresh5 = (*mtx
        .offset(2 as i32 as isize))[0 as i32 as usize];
    *fresh5 *= (*pp).size;
    let ref mut fresh6 = (*mtx
        .offset(0 as i32 as isize))[1 as i32 as usize];
    *fresh6 *= (*pp).size;
    let ref mut fresh7 = (*mtx
        .offset(1 as i32 as isize))[1 as i32 as usize];
    *fresh7 *= (*pp).size;
    let ref mut fresh8 = (*mtx
        .offset(2 as i32 as isize))[1 as i32 as usize];
    *fresh8 *= (*pp).size;
    let ref mut fresh9 = (*mtx
        .offset(0 as i32 as isize))[2 as i32 as usize];
    *fresh9 *= (*pp).size;
    let ref mut fresh10 = (*mtx
        .offset(1 as i32 as isize))[2 as i32 as usize];
    *fresh10 *= (*pp).size;
    let ref mut fresh11 = (*mtx
        .offset(2 as i32 as isize))[2 as i32 as usize];
    *fresh11 *= (*pp).size;
}

pub unsafe fn psDispSubAPPSRTPoint(mut pp: *mut HSD_Particle) {
    melee_core::ct!("psDispSubAPPSRTPoint");
    let mut scratch_mtx: Mtx = [[0.; 4]; 3];
    let mut scratch_scale: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut cur_x: f32 = 0.;
    let mut cur_y: f32 = 0.;
    let mut cur_z: f32 = 0.;
    let mut prev_x: f32 = 0.;
    let mut prev_y: f32 = 0.;
    let mut prev_z: f32 = 0.;
    let mut w: u8 = 0;
    psSetCurrentMtx(GX_PNMTX1);
    if !(*pp).appsrt.is_null() {
        if (*(*pp).appsrt).frameNum as i32
            != psFrameNum as i32
        {
            let mut scale_x: f32 = 0.;
            let mut scale_y: f32 = 0.;
            if (*(*pp).appsrt).status as i32
                != PS_APPSTATUS_STILL as i32
            {
                let mut translate: *mut Vec3 = &raw mut (*(*pp).appsrt).translate;
                let mut rotate: *mut Vec3 = &raw mut (*(*pp).appsrt).rot as *mut Vec3;
                let mut scale: *mut Vec3 = &raw mut (*(*pp).appsrt).scale;
                let mut mmtx: MtxPtr = &raw mut (*(*pp).appsrt).mmtx as MtxPtr;
                HSD_MtxSRT(
                    mmtx as *mut [f32; 4],
                    scale,
                    rotate,
                    translate,
                    ::core::ptr::null_mut::<Vec3>(),
                );
            }
            if (*(*pp).appsrt).status as i32
                == PS_APPSTATUS_ONCE as i32
            {
                (*(*pp).appsrt).status = PS_APPSTATUS_STILL as i32
                    as i32 as u8;
            }
            PSMTXConcat(
                &raw mut vmtx as *mut [f32; 4],
                &raw mut (*(*pp).appsrt).mmtx as *mut [f32; 4],
                &raw mut (*(*pp).appsrt).ssx as *mut [f32; 4],
            );
            scale_x = fmadds(
                (*(*pp).appsrt).x84 as f32,
                (*(*pp).appsrt).x84 as f32,
                fmadds(
                    (*(*pp).appsrt).ssx,
                    (*(*pp).appsrt).ssx,
                    (*(*pp).appsrt).x74 as f32
                        * (*(*pp).appsrt).x74 as f32,
                ),
            ) as f32;
            scale_x = sqrtf(scale_x as f32) as f32;
            (*(*pp).appsrt).x94 = scale_x;
            scale_y = fmadds(
                (*(*pp).appsrt).x88 as f32,
                (*(*pp).appsrt).x88 as f32,
                fmadds(
                    (*(*pp).appsrt).ssy,
                    (*(*pp).appsrt).ssy,
                    (*(*pp).appsrt).x78 as f32
                        * (*(*pp).appsrt).x78 as f32,
                ),
            ) as f32;
            scale_y = sqrtf(scale_y as f32) as f32;
            (*(*pp).appsrt).x98 = scale_y;
            if (*(*pp).appsrt).xA2 as i32 != 0 as i32 {
                PSMTXIdentity(&raw mut scratch_mtx as *mut [f32; 4]);
                scratch_mtx[0 as i32
                    as usize][3 as i32 as usize] = (*(*pp).appsrt)
                    .translate
                    .x;
                scratch_mtx[1 as i32
                    as usize][3 as i32 as usize] = (*(*pp).appsrt)
                    .translate
                    .y;
                scratch_mtx[2 as i32
                    as usize][3 as i32 as usize] = (*(*pp).appsrt)
                    .translate
                    .z;
                PSMTXConcat(
                    &raw mut vmtx as *mut [f32; 4],
                    &raw mut scratch_mtx as *mut [f32; 4],
                    &raw mut scratch_mtx as *mut [f32; 4],
                );
                HSD_MtxGetScale(
                    &raw mut scratch_mtx as *mut [f32; 4],
                    &raw mut scratch_scale,
                );
                PSMTXScale(
                    &raw mut (*(*pp).appsrt).ssx as *mut [f32; 4],
                    scratch_scale.x,
                    scratch_scale.y,
                    scratch_scale.z,
                );
                (*(*pp).appsrt).x70 = scratch_mtx[0 as i32
                    as usize][3 as i32 as usize];
                (*(*pp).appsrt).x80 = scratch_mtx[1 as i32
                    as usize][3 as i32 as usize];
                (*(*pp).appsrt).x90 = scratch_mtx[2 as i32
                    as usize][3 as i32 as usize];
            }
        }
        (*(*pp).appsrt).frameNum = psFrameNum;
    }
    cur_x = ((*(*pp).appsrt).x70 as f32
        + fmadds(
            (*(*pp).appsrt).x6C as f32,
            (*pp).pos.z as f32,
            fmadds(
                (*(*pp).appsrt).ssx,
                (*pp).pos.x as f32,
                (*(*pp).appsrt).ssy * (*pp).pos.y as f32,
            ),
        )) as f32;
    cur_y = ((*(*pp).appsrt).x80 as f32
        + fmadds(
            (*(*pp).appsrt).x7C as f32,
            (*pp).pos.z as f32,
            fmadds(
                (*(*pp).appsrt).x74 as f32,
                (*pp).pos.x as f32,
                (*(*pp).appsrt).x78 as f32
                    * (*pp).pos.y as f32,
            ),
        )) as f32;
    cur_z = ((*(*pp).appsrt).x90 as f32
        + fmadds(
            (*(*pp).appsrt).x8C as f32,
            (*pp).pos.z as f32,
            fmadds(
                (*(*pp).appsrt).x84 as f32,
                (*pp).pos.x as f32,
                (*(*pp).appsrt).x88 as f32
                    * (*pp).pos.y as f32,
            ),
        )) as f32;
    if (*pp).kind & Tornado as i32 as u32 != 0 {
        let mut x: f32 = 0.;
        let mut y: f32 = 0.;
        let mut z: f32 = 0.;
        calcTornadoLastPos(pp, &raw mut x, &raw mut y, &raw mut z);
        prev_x = ((*(*pp).appsrt).x70 as f32
            + fmadds(
                (*(*pp).appsrt).x6C as f32,
                z as f32,
                fmadds(
                    (*(*pp).appsrt).ssx,
                    x as f32,
                    (*(*pp).appsrt).ssy * y as f32,
                ),
            )) as f32;
        prev_y = ((*(*pp).appsrt).x80 as f32
            + fmadds(
                (*(*pp).appsrt).x7C as f32,
                z as f32,
                fmadds(
                    (*(*pp).appsrt).x74 as f32,
                    x as f32,
                    (*(*pp).appsrt).x78 as f32
                        * y as f32,
                ),
            )) as f32;
        prev_z = ((*(*pp).appsrt).x90 as f32
            + fmadds(
                (*(*pp).appsrt).x8C as f32,
                z as f32,
                fmadds(
                    (*(*pp).appsrt).x84 as f32,
                    x as f32,
                    (*(*pp).appsrt).x88 as f32
                        * y as f32,
                ),
            )) as f32;
    } else {
        prev_x = ((*(*pp).appsrt).x70 as f32
            + fmadds(
                (*(*pp).appsrt).x6C as f32,
                (*pp).pos.z as f32
                    - (*pp).vel.z as f32,
                fmadds(
                    (*(*pp).appsrt).ssx,
                    (*pp).pos.x as f32
                        - (*pp).vel.x as f32,
                    (*(*pp).appsrt).ssy
                        * ((*pp).pos.y as f32
                            - (*pp).vel.y as f32),
                ),
            )) as f32;
        prev_y = ((*(*pp).appsrt).x80 as f32
            + fmadds(
                (*(*pp).appsrt).x7C as f32,
                (*pp).pos.z as f32
                    - (*pp).vel.z as f32,
                fmadds(
                    (*(*pp).appsrt).x74 as f32,
                    (*pp).pos.x as f32
                        - (*pp).vel.x as f32,
                    (*(*pp).appsrt).x78 as f32
                        * ((*pp).pos.y as f32
                            - (*pp).vel.y as f32),
                ),
            )) as f32;
        prev_z = ((*(*pp).appsrt).x90 as f32
            + fmadds(
                (*(*pp).appsrt).x8C as f32,
                (*pp).pos.z as f32
                    - (*pp).vel.z as f32,
                fmadds(
                    (*(*pp).appsrt).x84 as f32,
                    (*pp).pos.x as f32
                        - (*pp).vel.x as f32,
                    (*(*pp).appsrt).x88 as f32
                        * ((*pp).pos.y as f32
                            - (*pp).vel.y as f32),
                ),
            )) as f32;
    }
    w = (if (*pp).size as f64 > 42.5f64 {
        255.0f32
    } else {
        6.0f32 * (*pp).size
    }) as i32 as u8;
    if (*pp).kind & Trail as i32 as u32 != 0 {
        let mut draw_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
        if prevLineWidth != w as s32 {
            prevLineWidth = w as s32;
            GXSetLineWidth(w, GX_TO_ONE);
        }
        getClrTrail(pp, &raw mut draw_color);
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            setVtxDesc(2 as s32);
            GXBegin(GX_LINES, GX_VTXFMT2, 2 as u16);
        } else {
            setVtxDesc(3 as s32);
            GXBegin(GX_LINES, GX_VTXFMT3, 2 as u16);
        }
        GXPosition3f32(prev_x, prev_y, prev_z);
        GXColor4u8(
            draw_color.r,
            draw_color.g,
            draw_color.b,
            (draw_color.a as f32 * (*pp).trail) as i32 as u8,
        );
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            GXTexCoord1x8(0 as i32 as u8);
        }
        GXPosition3f32(cur_x, cur_y, cur_z);
        GXColor4u8(draw_color.r, draw_color.g, draw_color.b, draw_color.a);
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            GXTexCoord1x8(1 as i32 as u8);
        }
    } else {
        if prevPointSize != w as s32 {
            prevPointSize = w as s32;
            GXSetPointSize(w, GX_TO_ONE);
        }
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            setVtxDesc(0 as s32);
            GXBegin(GX_POINTS, GX_VTXFMT0, 1 as u16);
        } else {
            setVtxDesc(1 as s32);
            GXBegin(GX_POINTS, GX_VTXFMT1, 1 as u16);
        }
        GXPosition3f32(cur_x, cur_y, cur_z);
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            GXTexCoord1x8(1 as i32 as u8);
        }
    };
}

pub unsafe fn psDispSubAppSRT(mut pp: *mut HSD_Particle, mut texform: *mut u8) {
    melee_core::ct!("psDispSubAppSRT");
    let mut draw_mtx: Mtx = [[0.; 4]; 3];
    let mut pad: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut scratch_scale: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut x_extent: f32 = 0.;
    let mut y_extent: f32 = 0.;
    let mut ax: f32 = 0.;
    let mut ay: f32 = 0.;
    let mut bx: f32 = 0.;
    let mut by: f32 = 0.;
    let mut angle: f32 = 0.;
    let mut it: *mut u8 = texform;
    let mut w0: f32 = 0.;
    let mut w1: f32 = 0.;
    let mut w0inv: f32 = 0.;
    let mut w1inv: f32 = 0.;
    let mut f11: f32 = 0.;
    let mut f8: f32 = 0.;
    let mut f12: f32 = 0.;
    let mut f13: f32 = 0.;
    let mut f16: f32 = 0.;
    let mut f20: f32 = 0.;
    let mut f17: f32 = 0.;
    let mut f18: f32 = 0.;
    let mut f20b: f32 = 0.;
    let mut s7F8: f32 = 0.;
    let mut s7FC: f32 = 0.;
    let mut s800: f32 = 0.;
    let mut s804: f32 = 0.;
    let mut s808: f32 = 0.;
    let mut prev_pos_x: f32 = 0.;
    let mut prev_pos_y: f32 = 0.;
    let mut prev_pos_z: f32 = 0.;
    let mut cur_x: f32 = 0.;
    let mut cur_y: f32 = 0.;
    let mut cur_z: f32 = 0.;
    if (*(*pp).appsrt).frameNum as i32 != psFrameNum as i32
    {
        let mut scale_x: f32 = 0.;
        let mut scale_y: f32 = 0.;
        if (*(*pp).appsrt).status as i32
            != PS_APPSTATUS_STILL as i32
        {
            let mut translate: *mut Vec3 = &raw mut (*(*pp).appsrt).translate;
            let mut rotate: *mut Vec3 = &raw mut (*(*pp).appsrt).rot as *mut Vec3;
            let mut scale: *mut Vec3 = &raw mut (*(*pp).appsrt).scale;
            let mut mmtx: MtxPtr = &raw mut (*(*pp).appsrt).mmtx as MtxPtr;
            HSD_MtxSRT(
                mmtx as *mut [f32; 4],
                scale,
                rotate,
                translate,
                ::core::ptr::null_mut::<Vec3>(),
            );
        }
        if (*(*pp).appsrt).status as i32
            == PS_APPSTATUS_ONCE as i32
        {
            (*(*pp).appsrt).status = PS_APPSTATUS_STILL as i32 as i32 as u8;
        }
        PSMTXConcat(
            &raw mut vmtx as *mut [f32; 4],
            &raw mut (*(*pp).appsrt).mmtx as *mut [f32; 4],
            &raw mut (*(*pp).appsrt).ssx as *mut [f32; 4],
        );
        scale_x = fmadds(
            (*(*pp).appsrt).x84 as f32,
            (*(*pp).appsrt).x84 as f32,
            fmadds(
                (*(*pp).appsrt).ssx,
                (*(*pp).appsrt).ssx,
                (*(*pp).appsrt).x74 as f32
                    * (*(*pp).appsrt).x74 as f32,
            ),
        ) as f32;
        scale_x = sqrtf(scale_x as f32) as f32;
        (*(*pp).appsrt).x94 = scale_x;
        scale_y = fmadds(
            (*(*pp).appsrt).x88 as f32,
            (*(*pp).appsrt).x88 as f32,
            fmadds(
                (*(*pp).appsrt).ssy,
                (*(*pp).appsrt).ssy,
                (*(*pp).appsrt).x78 as f32
                    * (*(*pp).appsrt).x78 as f32,
            ),
        ) as f32;
        scale_y = sqrtf(scale_y as f32) as f32;
        (*(*pp).appsrt).x98 = scale_y;
        if (*(*pp).appsrt).xA2 as i32 != 0 as i32 {
            PSMTXIdentity(&raw mut draw_mtx as *mut [f32; 4]);
            draw_mtx[0 as i32
                as usize][3 as i32 as usize] = (*(*pp).appsrt)
                .translate
                .x;
            draw_mtx[1 as i32
                as usize][3 as i32 as usize] = (*(*pp).appsrt)
                .translate
                .y;
            draw_mtx[2 as i32
                as usize][3 as i32 as usize] = (*(*pp).appsrt)
                .translate
                .z;
            PSMTXConcat(
                &raw mut vmtx as *mut [f32; 4],
                &raw mut draw_mtx as *mut [f32; 4],
                &raw mut draw_mtx as *mut [f32; 4],
            );
            HSD_MtxGetScale(
                &raw mut draw_mtx as *mut [f32; 4],
                &raw mut scratch_scale,
            );
            PSMTXScale(
                &raw mut (*(*pp).appsrt).ssx as *mut [f32; 4],
                scratch_scale.x,
                scratch_scale.y,
                scratch_scale.z,
            );
            (*(*pp).appsrt).x70 = draw_mtx[0 as i32
                as usize][3 as i32 as usize];
            (*(*pp).appsrt).x80 = draw_mtx[1 as i32
                as usize][3 as i32 as usize];
            (*(*pp).appsrt).x90 = draw_mtx[2 as i32
                as usize][3 as i32 as usize];
        }
        (*(*pp).appsrt).frameNum = psFrameNum;
    }
    let mut pos_x: f32 = 0.;
    let mut pos_y: f32 = 0.;
    PSMTXCopy(
        &raw mut (*(*pp).appsrt).ssx as *mut [f32; 4],
        &raw mut draw_mtx as *mut [f32; 4],
    );
    pos_x = (*pp).pos.x;
    pos_y = (*pp).pos.y;
    cur_x = (draw_mtx[0 as i32 as usize][3 as i32 as usize]
        + fmadds(
            draw_mtx[0 as i32 as usize][2 as i32 as usize],
            (*pp).pos.z as f32,
            fmadds(
                draw_mtx[0 as i32
                    as usize][0 as i32 as usize],
                pos_x,
                draw_mtx[0 as i32
                    as usize][1 as i32 as usize] * pos_y,
            ),
        )) as f32;
    cur_y = (draw_mtx[1 as i32 as usize][3 as i32 as usize]
        + fmadds(
            draw_mtx[1 as i32 as usize][2 as i32 as usize],
            (*pp).pos.z as f32,
            fmadds(
                draw_mtx[1 as i32
                    as usize][0 as i32 as usize],
                pos_x as f32,
                draw_mtx[1 as i32
                    as usize][1 as i32 as usize]
                    * pos_y as f32,
            ),
        )) as f32;
    cur_z = (draw_mtx[2 as i32 as usize][3 as i32 as usize]
        + fmadds(
            draw_mtx[2 as i32 as usize][2 as i32 as usize],
            (*pp).pos.z as f32,
            fmadds(
                draw_mtx[2 as i32
                    as usize][0 as i32 as usize],
                pos_x as f32,
                draw_mtx[2 as i32
                    as usize][1 as i32 as usize]
                    * pos_y as f32,
            ),
        )) as f32;
    if (*pp).kind & Tornado as i32 as u32 != 0 {
        let mut x: f32 = 0.;
        let mut y: f32 = 0.;
        let mut z: f32 = 0.;
        calcTornadoLastPos(pp, &raw mut x, &raw mut y, &raw mut z);
        prev_pos_x = (draw_mtx[0 as i32
            as usize][3 as i32 as usize]
            + fmadds(
                draw_mtx[0 as i32
                    as usize][2 as i32 as usize],
                z as f32,
                fmadds(
                    draw_mtx[0 as i32
                        as usize][0 as i32 as usize],
                    x as f32,
                    draw_mtx[0 as i32
                        as usize][1 as i32 as usize]
                        * y as f32,
                ),
            )) as f32;
        prev_pos_y = (draw_mtx[1 as i32
            as usize][3 as i32 as usize]
            + fmadds(
                draw_mtx[1 as i32
                    as usize][2 as i32 as usize],
                z as f32,
                fmadds(
                    draw_mtx[1 as i32
                        as usize][0 as i32 as usize],
                    x as f32,
                    draw_mtx[1 as i32
                        as usize][1 as i32 as usize]
                        * y as f32,
                ),
            )) as f32;
        prev_pos_z = (draw_mtx[2 as i32
            as usize][3 as i32 as usize]
            + fmadds(
                draw_mtx[2 as i32
                    as usize][2 as i32 as usize],
                z as f32,
                fmadds(
                    draw_mtx[2 as i32
                        as usize][0 as i32 as usize],
                    x as f32,
                    draw_mtx[2 as i32
                        as usize][1 as i32 as usize]
                        * y as f32,
                ),
            )) as f32;
    } else {
        let mut dz: f32 = (*pp).pos.z - (*pp).vel.z;
        let mut dx: f32 = (*pp).pos.x - (*pp).vel.x;
        let mut dy: f32 = (*pp).pos.y - (*pp).vel.y;
        prev_pos_x = (draw_mtx[0 as i32
            as usize][3 as i32 as usize]
            + fmadds(
                draw_mtx[0 as i32
                    as usize][2 as i32 as usize],
                dz as f32,
                fmadds(
                    draw_mtx[0 as i32
                        as usize][0 as i32 as usize],
                    dx as f32,
                    draw_mtx[0 as i32
                        as usize][1 as i32 as usize]
                        * dy as f32,
                ),
            )) as f32;
        prev_pos_y = (draw_mtx[1 as i32
            as usize][3 as i32 as usize]
            + fmadds(
                draw_mtx[1 as i32
                    as usize][2 as i32 as usize],
                dz as f32,
                fmadds(
                    draw_mtx[1 as i32
                        as usize][0 as i32 as usize],
                    dx as f32,
                    draw_mtx[1 as i32
                        as usize][1 as i32 as usize]
                        * dy as f32,
                ),
            )) as f32;
        prev_pos_z = (draw_mtx[2 as i32
            as usize][3 as i32 as usize]
            + fmadds(
                draw_mtx[2 as i32
                    as usize][2 as i32 as usize],
                dz as f32,
                fmadds(
                    draw_mtx[2 as i32
                        as usize][0 as i32 as usize],
                    dx as f32,
                    draw_mtx[2 as i32
                        as usize][1 as i32 as usize]
                        * dy as f32,
                ),
            )) as f32;
    }
    psScaleAppSRTAxes(pp, &raw mut draw_mtx as *mut [f32; 4]);
    let _ = &raw mut pad;
    x_extent = ((*(*pp).appsrt).x94 as f32 * (*pp).size) as f32;
    y_extent = ((*(*pp).appsrt).x98 as f32 * (*pp).size) as f32;
    if it.is_null() {
        ay = y_extent;
        by = -ay;
        bx = x_extent;
        ax = bx;
    } else {
        ay = 0.0f32 as f32;
        ax = x_extent;
        by = -y_extent;
        bx = ay;
    }
    if (*pp).kind & Trail as i32 as u32 != 0
        || (*pp).kind & DirVec as i32 as u32 != 0
    {
        let mut vf1: f32 = 0.;
        let mut vf2: f32 = 0.;
        let mut m00: f32 = (*(*pp).appsrt).ssx as f32;
        let mut m01: f32 = (*(*pp).appsrt).ssy as f32;
        let mut m02: f32 = (*(*pp).appsrt).x6C;
        let mut m03: f32 = (*(*pp).appsrt).x70;
        let mut m10: f32 = (*(*pp).appsrt).x74;
        let mut m11: f32 = (*(*pp).appsrt).x78;
        let mut m12: f32 = (*(*pp).appsrt).x7C;
        let mut m13: f32 = (*(*pp).appsrt).x80;
        let mut m20: f32 = (*(*pp).appsrt).x84;
        let mut m21: f32 = (*(*pp).appsrt).x88;
        let mut m22: f32 = (*(*pp).appsrt).x8C;
        let mut m23: f32 = (*(*pp).appsrt).x90;
        if 0.0f32 == prj[0 as i32 as usize] {
            let mut prev_x: f32 = 0.;
            let mut prev_y: f32 = 0.;
            let mut prev_z: f32 = 0.;
            if (*pp).kind & Tornado as i32 as u32 != 0 {
                calcTornadoLastPos(
                    pp,
                    &raw mut prev_x,
                    &raw mut prev_y,
                    &raw mut prev_z,
                );
            } else {
                prev_x = (*pp).pos.x - (*pp).vel.x;
                prev_y = (*pp).pos.y - (*pp).vel.y;
                prev_z = (*pp).pos.z - (*pp).vel.z;
            }
            w0 = ((*(*pp).appsrt).x90 as f32
                + fmadds(
                    (*(*pp).appsrt).x8C as f32,
                    (*pp).pos.z as f32,
                    fmadds(
                        (*(*pp).appsrt).x84 as f32,
                        (*pp).pos.x as f32,
                        (*(*pp).appsrt).x88 as f32
                            * (*pp).pos.y as f32,
                    ),
                )) as f32;
            s808 = fmadds(
                prj[1 as i32 as usize],
                (*(*pp).appsrt).ssx,
                prj[2 as i32 as usize]
                    * (*(*pp).appsrt).x84 as f32,
            ) as f32;
            s804 = fmadds(
                prj[1 as i32 as usize],
                (*(*pp).appsrt).ssy,
                prj[2 as i32 as usize]
                    * (*(*pp).appsrt).x88 as f32,
            ) as f32;
            f16 = fmadds(
                prj[1 as i32 as usize],
                (*(*pp).appsrt).x6C as f32,
                prj[2 as i32 as usize]
                    * (*(*pp).appsrt).x8C as f32,
            ) as f32;
            f20 = fmadds(
                prj[1 as i32 as usize],
                (*(*pp).appsrt).x70 as f32,
                prj[2 as i32 as usize]
                    * (*(*pp).appsrt).x90 as f32,
            ) as f32;
            f12 = fmadds(
                prj[3 as i32 as usize],
                (*(*pp).appsrt).x74 as f32,
                prj[4 as i32 as usize]
                    * (*(*pp).appsrt).x84 as f32,
            ) as f32;
            f8 = fmadds(
                prj[3 as i32 as usize],
                (*(*pp).appsrt).x78 as f32,
                prj[4 as i32 as usize]
                    * (*(*pp).appsrt).x88 as f32,
            ) as f32;
            f11 = fmadds(
                prj[3 as i32 as usize],
                (*(*pp).appsrt).x7C as f32,
                prj[4 as i32 as usize]
                    * (*(*pp).appsrt).x8C as f32,
            ) as f32;
            f13 = fmadds(
                prj[3 as i32 as usize],
                (*(*pp).appsrt).x80 as f32,
                prj[4 as i32 as usize]
                    * (*(*pp).appsrt).x90 as f32,
            ) as f32;
            if 0.0f32 == w0 {
                return;
            }
            w0inv = -1.0f32 / w0;
            w1 = ((*(*pp).appsrt).x90 as f32
                + fmadds(
                    (*(*pp).appsrt).x8C as f32,
                    prev_z as f32,
                    fmadds(
                        (*(*pp).appsrt).x84 as f32,
                        prev_x as f32,
                        (*(*pp).appsrt).x88 as f32
                            * prev_y as f32,
                    ),
                )) as f32;
            if 0.0f32 == w1 {
                return;
            }
            w1inv = -1.0f32 / w1;
            vf1 = fmadds(
                w0inv as f32,
                f20 as f32
                    + fmadds(
                        f16 as f32,
                        (*pp).pos.z as f32,
                        fmadds(
                            s808 as f32,
                            (*pp).pos.x as f32,
                            s804 as f32
                                * (*pp).pos.y as f32,
                        ),
                    ),
                -(w1inv as f32
                    * (f20 as f32
                        + fmadds(
                            f16 as f32,
                            prev_z as f32,
                            fmadds(
                                s808 as f32,
                                prev_x as f32,
                                s804 as f32
                                    * prev_y as f32,
                            ),
                        ))),
            ) as f32;
            vf2 = fmadds(
                w0inv as f32,
                f13 as f32
                    + fmadds(
                        f11 as f32,
                        (*pp).pos.z as f32,
                        fmadds(
                            f12 as f32,
                            (*pp).pos.x as f32,
                            f8 as f32
                                * (*pp).pos.y as f32,
                        ),
                    ),
                -(w1inv as f32
                    * (f13 as f32
                        + fmadds(
                            f11 as f32,
                            prev_z as f32,
                            fmadds(
                                f12 as f32,
                                prev_x as f32,
                                f8 as f32 * prev_y as f32,
                            ),
                        ))),
            ) as f32;
        } else {
            s800 = fmadds(
                prj[1 as i32 as usize],
                (*(*pp).appsrt).ssx,
                prj[2 as i32 as usize],
            ) as f32;
            s7FC = fmadds(
                prj[1 as i32 as usize],
                (*(*pp).appsrt).ssy,
                prj[2 as i32 as usize],
            ) as f32;
            s7F8 = fmadds(
                prj[1 as i32 as usize],
                (*(*pp).appsrt).x6C as f32,
                prj[2 as i32 as usize],
            ) as f32;
            f17 = fmadds(
                prj[3 as i32 as usize],
                (*(*pp).appsrt).x74 as f32,
                prj[4 as i32 as usize],
            ) as f32;
            f18 = fmadds(
                prj[3 as i32 as usize],
                (*(*pp).appsrt).x78 as f32,
                prj[4 as i32 as usize],
            ) as f32;
            f20b = fmadds(
                prj[3 as i32 as usize],
                (*(*pp).appsrt).x7C as f32,
                prj[4 as i32 as usize],
            ) as f32;
            if (*pp).kind & Tornado as i32 as u32 != 0 {
                let mut tx: f32 = 0.;
                let mut ty: f32 = 0.;
                let mut tz: f32 = 0.;
                calcTornadoLastPos(pp, &raw mut tx, &raw mut ty, &raw mut tz);
                let mut dx_0: f32 = 0.;
                let mut dy_0: f32 = 0.;
                let mut dz_0: f32 = 0.;
                dy_0 = (*pp).pos.y - ty;
                dx_0 = (*pp).pos.x - tx;
                dz_0 = (*pp).pos.z - tz;
                vf1 = fmadds(
                    s7F8 as f32,
                    dz_0 as f32,
                    fmadds(
                        s800 as f32,
                        dx_0 as f32,
                        s7FC as f32 * dy_0 as f32,
                    ),
                ) as f32;
                vf2 = fmadds(
                    f20b as f32,
                    dz_0 as f32,
                    fmadds(
                        f17 as f32,
                        dx_0 as f32,
                        f18 as f32 * dy_0 as f32,
                    ),
                ) as f32;
            } else {
                let mut vz: f32 = 0.;
                let mut vx: f32 = 0.;
                let mut vy: f32 = 0.;
                vy = (*pp).vel.y;
                vx = (*pp).vel.x;
                vz = (*pp).vel.z;
                vf1 = fmadds(
                    s7F8 as f32,
                    vz as f32,
                    fmadds(
                        s800 as f32,
                        vx as f32,
                        s7FC as f32 * vy as f32,
                    ),
                ) as f32;
                vf2 = fmadds(
                    f20b as f32,
                    vz as f32,
                    fmadds(
                        f17 as f32,
                        vx as f32,
                        f18 as f32 * vy as f32,
                    ),
                ) as f32;
            }
        }
        if psMaskAbsLtF32(vf2, FLT_MIN) != 0 {
            angle = (if -vf1 >= 0.0f32 { 1.5707964f32 } else { -1.5707964f32 }) as f32;
        } else {
            angle = atan2f(-(vf1 as f32), vf2 as f32)
                as f32;
        }
        if (*pp).kind & DirVec as i32 as u32 != 0 {
            angle += (*pp).rotate;
        }
    } else {
        angle = (*pp).rotate as f32;
    }
    if psMaskAbsGtF32(angle, 0.01f64) != 0 {
        let mut c: f32 = cosf(angle as f32) as f32;
        let mut s: f32 = sinf(angle as f32) as f32;
        let mut old_x: f32 = ax;
        ax = fmadds(
            c as f32,
            ax as f32,
            -(s as f32 * ay as f32),
        ) as f32;
        ay = fmadds(
            s as f32,
            old_x as f32,
            c as f32 * ay as f32,
        ) as f32;
        old_x = bx;
        bx = fmadds(
            c as f32,
            bx as f32,
            -(s as f32 * by as f32),
        ) as f32;
        by = fmadds(
            s as f32,
            old_x as f32,
            c as f32 * by as f32,
        ) as f32;
    }
    psSetCurrentMtx(GX_PNMTX1);
    if (*pp).kind & Trail as i32 as u32 != 0 {
        let mut xl: f32 = 0.;
        let mut yl: f32 = 0.;
        let mut zl: f32 = 0.;
        let mut draw_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
        getClrTrail(pp, &raw mut draw_color);
        if it.is_null() {
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                setVtxDesc(2 as s32);
                GXBegin(GX_QUADS, GX_VTXFMT2, 4 as u16);
            } else {
                setVtxDesc(3 as s32);
                GXBegin(GX_QUADS, GX_VTXFMT3, 4 as u16);
            }
            GXPosition3f32(-ax + prev_pos_x, -ay + prev_pos_y, prev_pos_z);
            GXColor4u8(
                draw_color.r,
                draw_color.g,
                draw_color.b,
                (draw_color.a as f32 * (*pp).trail) as i32 as u8,
            );
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                GXTexCoord1x8(
                    ((*pp).kind >> 16 as i32 & 0xc as u32) as i32 as u8,
                );
            }
            GXPosition3f32(-bx + cur_x, -by + cur_y, cur_z);
            GXColor4u8(draw_color.r, draw_color.g, draw_color.b, draw_color.a);
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                GXTexCoord1x8(
                    ((*pp).kind >> 16 as i32 & 0xc as u32)
                        .wrapping_add(1 as u32) as i32 as u8,
                );
            }
            GXPosition3f32(ax + cur_x, ay + cur_y, cur_z);
            GXColor4u8(draw_color.r, draw_color.g, draw_color.b, draw_color.a);
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                GXTexCoord1x8(
                    ((*pp).kind >> 16 as i32 & 0xc as u32)
                        .wrapping_add(2 as u32) as i32 as u8,
                );
            }
            GXPosition3f32(bx + prev_pos_x, by + prev_pos_y, prev_pos_z);
            GXColor4u8(
                draw_color.r,
                draw_color.g,
                draw_color.b,
                (draw_color.a as f32 * (*pp).trail) as i32 as u8,
            );
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                GXTexCoord1x8(
                    ((*pp).kind >> 16 as i32 & 0xc as u32)
                        .wrapping_add(3 as u32) as i32 as u8,
                );
            }
        } else {
            let mut trail_alpha: f32 = 255.0f32 * (1.0f32 - (*pp).trail as f32);
            let mut axis_len: f32 = sqrtf(
                fmadds(
                    bx as f32,
                    bx as f32,
                    by as f32 * by as f32,
                ),
            ) as f32;
            if axis_len != 0.0f32 {
                let mut dx_1: f32 = cur_x - prev_pos_x;
                let mut dy_1: f32 = cur_y - prev_pos_y;
                let mut dz_1: f32 = cur_z - prev_pos_z;
                let mut primitive_count: u32 = 0;
                xl = dx_1 * dx_1;
                yl = dy_1 * dy_1;
                zl = dz_1 * dz_1;
                axis_len = sqrtf(
                    zl as f32
                        + (xl as f32 + yl as f32),
                ) as f32 / axis_len;
                primitive_count = *(it as *mut u32);
                bx *= axis_len;
                by *= axis_len;
                it = it.offset(::core::mem::size_of::<u32>() as usize as isize);
                while primitive_count != 0 as u32 {
                    let mut primitive: GXPrimitive = *it
                        .offset(0 as i32 as isize) as GXPrimitive;
                    let mut count: u8 = *it.offset(1 as i32 as isize);
                    let mut i: s32 = 0;
                    it = it.offset(4 as i32 as isize);
                    if (*pp).kind & DispTexture as i32 as u32 != 0 {
                        setVtxDesc(5 as s32);
                        GXBegin(primitive, GX_VTXFMT5, count as u16);
                    } else {
                        setVtxDesc(3 as s32);
                        GXBegin(primitive, GX_VTXFMT3, count as u16);
                    }
                    i = count as s32;
                    while i > 0 as i32 {
                        let mut s_0: f32 = *(it
                            .offset(0 as i32 as isize) as *mut u8
                            as *mut f32);
                        let mut sx: f32 = 2.0f32 * (s_0 - 0.5f32);
                        let mut t: f32 = 0.;
                        let mut tx_0: f32 = 0.;
                        let mut converted_alpha: s32 = 0;
                        let mut alpha: s32 = 0;
                        if (*pp).kind & TexFlipS as i32 as u32 != 0 {
                            s_0 = 1.0f32 - s_0;
                        }
                        t = *(it.offset(4 as i32 as isize) as *mut u8
                            as *mut f32);
                        it = it.offset(8 as i32 as isize);
                        alpha = fmadds(
                            -(t as f32),
                            trail_alpha as f32,
                            255.0f32,
                        ) as s32;
                        converted_alpha = fmadds(
                            -(t as f32),
                            trail_alpha as f32,
                            255.0f32,
                        ) as s32;
                        if converted_alpha < 0 as i32 {
                            alpha = 0 as i32 as s32;
                        }
                        if alpha > 0xff as i32 {
                            alpha = 0xff as i32 as s32;
                        }
                        tx_0 = (2.0f32 * (t as f32 - 0.5f32)) as f32;
                        if (*pp).kind & TexFlipT as i32 as u32 != 0 {
                            t = 1.0f32 - t;
                        }
                        GXPosition3f32(
                            fmadds(
                                bx as f32,
                                tx_0 as f32,
                                fmadds(
                                    ax as f32,
                                    sx as f32,
                                    cur_x as f32,
                                ),
                            ) as f32,
                            fmadds(
                                by as f32,
                                tx_0 as f32,
                                fmadds(
                                    ay as f32,
                                    sx as f32,
                                    cur_y as f32,
                                ),
                            ) as f32,
                            cur_z,
                        );
                        GXColor4u8(
                            draw_color.r,
                            draw_color.g,
                            draw_color.b,
                            alpha as i32 as u8,
                        );
                        if (*pp).kind & DispTexture as i32 as u32 != 0 {
                            GXTexCoord2f32(s_0, t);
                        }
                        i -= 1;
                    }
                    primitive_count = primitive_count.wrapping_sub(1);
                }
            }
        }
    } else if it.is_null() {
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            setVtxDesc(0 as s32);
            GXBegin(GX_QUADS, GX_VTXFMT0, 4 as u16);
        } else {
            setVtxDesc(1 as s32);
            GXBegin(GX_QUADS, GX_VTXFMT1, 4 as u16);
        }
        GXPosition3f32(-ax + cur_x, -ay + cur_y, cur_z);
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            GXTexCoord1x8(
                ((*pp).kind >> 16 as i32 & 0xc as u32) as i32 as u8,
            );
        }
        GXPosition3f32(-bx + cur_x, -by + cur_y, cur_z);
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            GXTexCoord1x8(
                ((*pp).kind >> 16 as i32 & 0xc as u32)
                    .wrapping_add(1 as u32) as i32 as u8,
            );
        }
        GXPosition3f32(ax + cur_x, ay + cur_y, cur_z);
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            GXTexCoord1x8(
                ((*pp).kind >> 16 as i32 & 0xc as u32)
                    .wrapping_add(2 as u32) as i32 as u8,
            );
        }
        GXPosition3f32(bx + cur_x, by + cur_y, cur_z);
        if (*pp).kind & DispTexture as i32 as u32 != 0 {
            GXTexCoord1x8(
                ((*pp).kind >> 16 as i32 & 0xc as u32)
                    .wrapping_add(3 as u32) as i32 as u8,
            );
        }
    } else {
        let mut primitive_count_0: u32 = *(it as *mut u32);
        it = it.offset(::core::mem::size_of::<u32>() as usize as isize);
        while primitive_count_0 != 0 as u32 {
            let mut prim: GXPrimitive = *it.offset(0 as i32 as isize)
                as GXPrimitive;
            let mut count_0: u8 = *it.offset(1 as i32 as isize);
            let mut i_0: s32 = 0;
            it = it.offset(4 as i32 as isize);
            if (*pp).kind & DispTexture as i32 as u32 != 0 {
                setVtxDesc(4 as s32);
                GXBegin(prim, GX_VTXFMT4, count_0 as u16);
            } else {
                setVtxDesc(1 as s32);
                GXBegin(prim, GX_VTXFMT1, count_0 as u16);
            }
            i_0 = count_0 as s32;
            while i_0 > 0 as i32 {
                let mut s_1: f32 = *(it.offset(0 as i32 as isize)
                    as *mut u8 as *mut f32);
                let mut sx_0: f32 = 2.0f32 * (s_1 - 0.5f32);
                let mut t_0: f32 = 0.;
                let mut tx_1: f32 = 0.;
                if (*pp).kind & TexFlipS as i32 as u32 != 0 {
                    s_1 = 1.0f32 - s_1;
                }
                t_0 = *(it.offset(4 as i32 as isize) as *mut u8
                    as *mut f32);
                it = it.offset(8 as i32 as isize);
                tx_1 = (2.0f32 * (t_0 as f32 - 0.5f32)) as f32;
                if (*pp).kind & TexFlipT as i32 as u32 != 0 {
                    t_0 = 1.0f32 - t_0;
                }
                GXPosition3f32(
                    fmadds(
                        bx as f32,
                        tx_1 as f32,
                        fmadds(
                            ax as f32,
                            sx_0 as f32,
                            cur_x as f32,
                        ),
                    ) as f32,
                    fmadds(
                        by as f32,
                        tx_1 as f32,
                        fmadds(
                            ay as f32,
                            sx_0 as f32,
                            cur_y as f32,
                        ),
                    ) as f32,
                    cur_z,
                );
                if (*pp).kind & DispTexture as i32 as u32 != 0 {
                    GXTexCoord2f32(s_1, t_0);
                }
                i_0 -= 1;
            }
            primitive_count_0 = primitive_count_0.wrapping_sub(1);
        }
    };
}

pub unsafe fn psUpdateBillboardAxes(mut inv_view: *const [f32; 4]) {
    melee_core::ct!("psUpdateBillboardAxes");
    let mut right_x: f32 = 0.;
    let mut up_x: f32 = 0.;
    let mut right_y: f32 = 0.;
    let mut up_y: f32 = 0.;
    let mut right_z: f32 = 0.;
    let mut up_z: f32 = 0.;
    right_x = (*inv_view
        .offset(0 as i32 as isize))[0 as i32 as usize];
    up_x = (*inv_view
        .offset(0 as i32 as isize))[1 as i32 as usize];
    HSD_PSDisp_804D7914 = right_x + up_x;
    HSD_PSDisp_804D7918 = right_x - up_x;
    right_y = (*inv_view
        .offset(1 as i32 as isize))[0 as i32 as usize];
    up_y = (*inv_view
        .offset(1 as i32 as isize))[1 as i32 as usize];
    HSD_PSDisp_804D791C = right_y + up_y;
    HSD_PSDisp_804D7920 = right_y - up_y;
    right_z = (*inv_view
        .offset(2 as i32 as isize))[0 as i32 as usize];
    up_z = (*inv_view
        .offset(2 as i32 as isize))[1 as i32 as usize];
    HSD_PSDisp_804D7924 = right_z + up_z;
    HSD_PSDisp_804D7928 = right_z - up_z;
}

pub unsafe fn psDispParticles(mut target_link: u32, mut sw: u32) {
    melee_core::ct!("psDispParticles");
    let mut sp7B4: s32 = 0;
    let mut sp7B0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
        ::core::ffi::c_void,
    >();
    let mut sp7AC: u32 = 0;
    let mut sp7A8: u32 = 0;
    let mut sp7A5: u8 = 0;
    let mut sp7A4: u8 = 0;
    let mut needs_setup: s32 = 0;
    let mut sp79C: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
        ::core::ffi::c_void,
    >();
    let mut tlut_obj: psdisp_Tlut = psdisp_Tlut {
        fmt: GX_TL_IA8,
        tlut_name: 0,
        n_entries: 0,
    };
    let mut stack_pad: s32 = 0;
    let mut sp764: GXTexObj = _GXTexObj { dummy: [0; 16] };
    let mut sorted_particles: *mut HSD_Particle = ::core::ptr::null_mut::<
        HSD_Particle,
    >();
    let mut non_edge_particles: *mut HSD_Particle = ::core::ptr::null_mut::<
        HSD_Particle,
    >();
    let mut billboard_mtx: psdisp_Mtx = psdisp_Mtx { mtx: [[0.; 4]; 3] };
    let mut y2: f32 = 0.;
    let mut gx_tlut_obj: GXTlutObj = _GXTlutObj { dummy: [0; 10] };
    let mut alpha_compare_mode: s32 = 0;
    let mut prev_tex_interp_near: s32 = 0;
    let mut prev_kind: u32 = 0;
    let mut pp: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    alpha_compare_mode = 0 as i32 as s32;
    prev_tex_interp_near = 0 as i32 as s32;
    sp7A5 = 0 as i32 as u8;
    sp7A4 = 0xff as i32 as u8;
    needs_setup = 1 as i32 as s32;
    if sw == 0 as u32 {
        if (psFrameNum as u32) < 0xff as u32 {
            psFrameNum = (psFrameNum as i32 + 1 as i32)
                as i32 as u8;
            return;
        }
        psFrameNum = 1 as i32 as u8;
        return;
    }
    sp7B4 = 0 as i32 as s32;
    loop {
        if target_link & ((1 as i32) << sp7B4) as u32 != 0 {
            particleSort(
                sp7B4,
                psFrameNum,
                &raw mut sorted_particles,
                &raw mut non_edge_particles,
            );
            if sw == 1 as u32 {
                pp = sorted_particles;
            } else {
                pp = non_edge_particles;
            }
            while !pp.is_null() {
                let mut tex_group: *mut HSD_PSTexGroup = ::core::ptr::null_mut::<
                    HSD_PSTexGroup,
                >();
                let mut form_group: *mut HSD_PSFormGroup = ::core::ptr::null_mut::<
                    HSD_PSFormGroup,
                >();
                let mut form: *mut u8 = ::core::ptr::null_mut::<u8>();
                let mut image: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
                    ::core::ffi::c_void,
                >();
                let mut tlut: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
                    ::core::ffi::c_void,
                >();
                let mut blend_mode: u32 = 0;
                let mut alpha0: u8 = 0;
                let mut alpha1: u8 = 0;
                let mut wrap_s: GXTexWrapMode = GX_CLAMP;
                let mut wrap_t: GXTexWrapMode = GX_CLAMP;
                let mut scale_s: f32 = 0.;
                let mut scale_t: f32 = 0.;
                let mut fmt: GXTexFmt = GX_TF_I4;
                let mut tex_table: *mut *mut u8 = ::core::ptr::null_mut::<*mut u8>();
                let mut width: u32 = 0;
                let mut height: u32 = 0;
                if sw == 1 as u32
                    && (*pp).kind & TexEdge as i32 as u32 == 0
                {
                    break;
                }
                if !((*pp).size < f32::EPSILON) {
                    if needs_setup != 0 as i32 {
                        sp79C = ::core::ptr::null_mut::<::core::ffi::c_void>();
                        prevPointSize = -(1 as i32) as s32;
                        sp7B0 = ::core::ptr::null_mut::<::core::ffi::c_void>();
                        prevLineWidth = -(1 as i32) as s32;
                        prevChanCtrl = -(1 as i32) as s32;
                        psSetupTevInvalidState();
                        sp7A8 = -(1 as i32) as u32;
                        prev_kind &= 0xfeffffff as u32;
                        sp7AC = -(1 as i32) as u32;
                        HSD_FogSet(::core::ptr::null_mut::<HSD_Fog>());
                        prevChanMat.b = 0xff as i32 as u8;
                        prevChanMat.g = prevChanMat.b;
                        prevChanMat.r = prevChanMat.g;
                        prevChanAmb.b = 0xff as i32 as u8;
                        prevChanAmb.g = prevChanAmb.b;
                        prevChanAmb.r = prevChanAmb.g;
                        prevChanAmb.a = 0xff as i32 as u8;
                        prevChanMat.a = prevChanAmb.a;
                        GXSetChanMatColor(GX_COLOR0A0, prevChanMat);
                        GXSetChanAmbColor(GX_COLOR0A0, prevChanAmb);
                        psSetupTevInvalidState();
                        psSetupTevCommon();
                        psSetColor(&raw mut prevColorPrim, 0xff as i32 as u8);
                        psSetColor(&raw mut prevColorEnv, 0 as i32 as u8);
                        psSetColor(&raw mut prevColorMat, 0xff as i32 as u8);
                        GXSetTevColor(GX_TEVREG0, prevColorPrim);
                        GXSetTevColor(GX_TEVREG1, prevColorEnv);
                        GXSetTevColor(GX_TEVREG2, prevColorMat);
                        HSD_PSDisp_804D792C = -(1 as i32) as s32;
                        GXSetZCompLoc(0);
                        HSD_CObjGetViewingMtx(
                            HSD_CObjGetCurrent(),
                            &raw mut vmtx as *mut [f32; 4],
                        );
                        PSMTXInverse(
                            &raw mut vmtx as *mut [f32; 4],
                            &raw mut rvmtx as *mut [f32; 4],
                        );
                        let mut w0: f32 = 0.;
                        let mut x_offset: f32 = 0.;
                        let mut x_scale: f32 = 0.;
                        let mut w1: f32 = 0.;
                        let mut y_offset: f32 = 0.;
                        let mut y_scale: f32 = 0.;
                        let mut w2: f32 = 0.;
                        let mut w3: f32 = 0.;
                        let mut y0: f32 = 0.;
                        let mut y1: f32 = 0.;
                        GXGetProjectionv(&raw mut prj as *mut f32);
                        if 0.0f32 == prj[0 as i32 as usize] {
                            x_scale = prj[1 as i32 as usize];
                            x_offset = prj[2 as i32 as usize];
                            w0 = vmtx[2 as i32
                                as usize][0 as i32 as usize];
                            pvmtx[0 as i32
                                as usize][0 as i32 as usize] = fmadds(
                                x_scale as f32,
                                vmtx[0 as i32
                                    as usize][0 as i32 as usize],
                                x_offset as f32
                                    * w0 as f32,
                            ) as f32;
                            w1 = vmtx[2 as i32
                                as usize][1 as i32 as usize];
                            pvmtx[0 as i32
                                as usize][1 as i32 as usize] = fmadds(
                                x_scale as f32,
                                vmtx[0 as i32
                                    as usize][1 as i32 as usize],
                                x_offset as f32
                                    * w1 as f32,
                            ) as f32;
                            w2 = vmtx[2 as i32
                                as usize][2 as i32 as usize];
                            pvmtx[0 as i32
                                as usize][2 as i32 as usize] = fmadds(
                                x_scale as f32,
                                vmtx[0 as i32
                                    as usize][2 as i32 as usize],
                                x_offset as f32
                                    * w2 as f32,
                            ) as f32;
                            w3 = vmtx[2 as i32
                                as usize][3 as i32 as usize];
                            pvmtx[0 as i32
                                as usize][3 as i32 as usize] = fmadds(
                                x_scale as f32,
                                vmtx[0 as i32
                                    as usize][3 as i32 as usize],
                                x_offset as f32
                                    * w3 as f32,
                            ) as f32;
                            y_scale = prj[3 as i32 as usize];
                            y_offset = prj[4 as i32 as usize];
                            y0 = y_offset * w0;
                            y1 = y_offset * w1;
                            y2 = y_offset * w2;
                            pvmtx[1 as i32
                                as usize][0 as i32 as usize] = fmadds(
                                y_scale as f32,
                                vmtx[1 as i32
                                    as usize][0 as i32 as usize],
                                y0 as f32,
                            ) as f32;
                            pvmtx[1 as i32
                                as usize][1 as i32 as usize] = fmadds(
                                y_scale as f32,
                                vmtx[1 as i32
                                    as usize][1 as i32 as usize],
                                y1 as f32,
                            ) as f32;
                            pvmtx[1 as i32
                                as usize][2 as i32 as usize] = fmadds(
                                y_scale as f32,
                                vmtx[1 as i32
                                    as usize][2 as i32 as usize],
                                y2 as f32,
                            ) as f32;
                            pvmtx[1 as i32
                                as usize][3 as i32 as usize] = fmadds(
                                y_scale as f32,
                                vmtx[1 as i32
                                    as usize][3 as i32 as usize],
                                y_offset as f32
                                    * w3 as f32,
                            ) as f32;
                        } else {
                            pvmtx[0 as i32
                                as usize][0 as i32 as usize] = fmadds(
                                prj[1 as i32 as usize],
                                vmtx[0 as i32
                                    as usize][0 as i32 as usize],
                                prj[2 as i32 as usize],
                            ) as f32;
                            pvmtx[0 as i32
                                as usize][1 as i32 as usize] = fmadds(
                                prj[1 as i32 as usize],
                                vmtx[0 as i32
                                    as usize][1 as i32 as usize],
                                prj[2 as i32 as usize],
                            ) as f32;
                            pvmtx[0 as i32
                                as usize][2 as i32 as usize] = fmadds(
                                prj[1 as i32 as usize],
                                vmtx[0 as i32
                                    as usize][2 as i32 as usize],
                                prj[2 as i32 as usize],
                            ) as f32;
                            pvmtx[0 as i32
                                as usize][3 as i32 as usize] = fmadds(
                                prj[1 as i32 as usize],
                                vmtx[0 as i32
                                    as usize][3 as i32 as usize],
                                prj[2 as i32 as usize],
                            ) as f32;
                            pvmtx[1 as i32
                                as usize][0 as i32 as usize] = fmadds(
                                prj[3 as i32 as usize],
                                vmtx[1 as i32
                                    as usize][0 as i32 as usize],
                                prj[4 as i32 as usize],
                            ) as f32;
                            pvmtx[1 as i32
                                as usize][1 as i32 as usize] = fmadds(
                                prj[3 as i32 as usize],
                                vmtx[1 as i32
                                    as usize][1 as i32 as usize],
                                prj[4 as i32 as usize],
                            ) as f32;
                            pvmtx[1 as i32
                                as usize][2 as i32 as usize] = fmadds(
                                prj[3 as i32 as usize],
                                vmtx[1 as i32
                                    as usize][2 as i32 as usize],
                                prj[4 as i32 as usize],
                            ) as f32;
                            pvmtx[1 as i32
                                as usize][3 as i32 as usize] = fmadds(
                                prj[3 as i32 as usize],
                                vmtx[1 as i32
                                    as usize][3 as i32 as usize],
                                prj[4 as i32 as usize],
                            ) as f32;
                        }
                        psUpdateBillboardAxes(
                            &raw const *(&raw mut rvmtx as *mut [f32; 4] as *const Mtx)
                                as *const [f32; 4],
                        );
                        GXLoadPosMtxImm(
                            &raw mut vmtx as *mut [f32; 4],
                            GX_PNMTX0 as i32 as u32,
                        );
                        billboard_mtx = HSD_PSDisp_803B9628;
                        GXLoadPosMtxImm(
                            &raw mut billboard_mtx.mtx as *mut [f32; 4],
                            GX_PNMTX1 as i32 as u32,
                        );
                        HSD_PSDisp_804D7948[0 as i32 as usize] = GX_PNMTX1
                            as i32 as s32;
                        psSetCurrentMtx(GX_PNMTX0);
                        GXEnableTexOffsets(GX_TEXCOORD0, 1, 1);
                        GXSetCullMode(GX_CULL_BACK);
                        GXSetArray(
                            GX_VA_TEX0,
                            &raw mut HSD_PSDisp_8040C340 as *mut u8
                                as *const ::core::ffi::c_void,
                            2 as i32 as u8,
                        );
                        psSetupVtxFormat(GX_VTXFMT0, 0, 1, GX_RGB565);
                        psSetupVtxFormat(GX_VTXFMT1, 0, 0, GX_RGB565);
                        psSetupVtxFormat(GX_VTXFMT2, 1, 1, GX_RGB565);
                        psSetupVtxFormat(GX_VTXFMT3, 1, 0, GX_RGB565);
                        psSetupVtxFormat(GX_VTXFMT4, 0, 1, GX_RGBA6);
                        psSetupVtxFormat(GX_VTXFMT5, 1, 1, GX_RGBA6);
                        needs_setup = 0 as i32 as s32;
                    }
                    blend_mode = (*pp).kind >> 0x16 as u32 & 3 as u32;
                    setBlendMode(blend_mode as i32);
                    if (*pp).aCmpCount as i32 != 0 as i32 {
                        let mut scale: s32 = 65536 as s32 * (*pp).aCmpRemain as s32
                            / (*pp).aCmpCount as s32;
                        alpha0 = ((((*pp).aCmpParam1Target as i32)
                            << 16 as i32)
                            + scale as i32
                                * ((*pp).aCmpParam1 as i32
                                    - (*pp).aCmpParam1Target as i32)
                            >> 16 as i32) as i32 as u8;
                        alpha1 = ((((*pp).aCmpParam2Target as i32)
                            << 16 as i32)
                            + scale as i32
                                * ((*pp).aCmpParam2 as i32
                                    - (*pp).aCmpParam2Target as i32)
                            >> 16 as i32) as i32 as u8;
                    } else {
                        alpha0 = (*pp).aCmpParam1;
                        alpha1 = (*pp).aCmpParam2;
                    }
                    if alpha_compare_mode != (*pp).aCmpMode as i32
                        || sp7A5 as i32 != alpha0 as i32
                        || sp7A4 as i32 != alpha1 as i32
                    {
                        sp7A5 = alpha0;
                        alpha_compare_mode = (*pp).aCmpMode as s32;
                        sp7A4 = alpha1;
                        GXSetAlphaCompare(
                            (alpha_compare_mode as i32
                                >> 3 as i32 & 7 as i32)
                                as GXCompare,
                            sp7A5,
                            (alpha_compare_mode as i32
                                >> 6 as i32 & 3 as i32)
                                as GXAlphaOp,
                            (alpha_compare_mode as i32
                                & 7 as i32) as GXCompare,
                            sp7A4,
                        );
                    }
                    psSetupTev(pp);
                    setupChanCtrl(pp);
                    setupChanReg(pp);
                    setupTevReg(pp);
                    if (*pp).kind & TexEdge as i32 as u32 != sp7AC {
                        sp7AC = (*pp).kind & TexEdge as i32 as u32;
                        if sp7AC as s32 != 0 as i32 {
                            GXSetZMode(1, GX_LEQUAL, 1);
                        } else {
                            GXSetZMode(1, GX_LEQUAL, 0);
                        }
                    }
                    if ((*pp).kind ^ prev_kind) & DispFog as i32 as u32
                        != 0 as u32
                    {
                        if (*pp).kind & DispFog as i32 as u32 != 0 {
                            HSD_FogSet(HSD_PSDisp_804D7908);
                        } else {
                            HSD_FogSet(::core::ptr::null_mut::<HSD_Fog>());
                        }
                    }
                    if !(*(&raw mut psNumCmdList as *mut *mut *mut HSD_PSFormGroup)
                        .offset((*pp).bank as isize))
                        .is_null()
                        && {
                            form_group = *(*(&raw mut psNumCmdList
                                as *mut *mut *mut HSD_PSFormGroup)
                                .offset((*pp).bank as isize))
                                .offset((*pp).texGroup as isize);
                            !form_group.is_null()
                        }
                    {
                        form = *(&raw mut (*form_group).formTable as *mut *mut u8)
                            .offset((*pp).poseNum as isize);
                    } else {
                        form = ::core::ptr::null_mut::<u8>();
                    }
                    if (*pp).kind & DispTexture as i32 as u32 != 0 {
                        if (*pp).kind & MirrorS as i32 as u32 != 0 {
                            scale_s = 2.0f32 as f32;
                            wrap_s = GX_MIRROR;
                        } else {
                            scale_s = 1.0f32 as f32;
                            wrap_s = GX_CLAMP;
                        }
                        if (*pp).kind & MirrorT as i32 as u32 != 0 {
                            scale_t = 2.0f32 as f32;
                            wrap_t = GX_MIRROR;
                        } else {
                            scale_t = 1.0f32 as f32;
                            wrap_t = GX_CLAMP;
                        }
                        if (*pp).kind
                            & (MirrorS as i32
                                | MirrorT as i32) as u32 != sp7A8
                        {
                            let mut temp_mtx: Mtx = [[0.; 4]; 3];
                            sp7A8 = (*pp).kind
                                & (MirrorS as i32
                                    | MirrorT as i32) as u32;
                            sp7B0 = ::core::ptr::null_mut::<::core::ffi::c_void>();
                            PSMTXScale(
                                &raw mut temp_mtx as *mut [f32; 4],
                                scale_s,
                                scale_t,
                                1.0f32,
                            );
                            if (*pp).kind & MirrorT as i32 as u32 != 0 {
                                temp_mtx[1 as i32
                                    as usize][3 as i32 as usize] = 1.0f32
                                    as f32;
                            }
                            GXLoadTexMtxImm(
                                &raw mut temp_mtx as *mut [f32; 4],
                                GX_TEXMTX0 as i32 as u32,
                                GX_MTX2x4,
                            );
                            GXSetTexCoordGen2(
                                GX_TEXCOORD0,
                                GX_TG_MTX2x4,
                                GX_TG_TEX0,
                                GX_TEXMTX0 as i32 as u32,
                                0,
                                GX_PTIDENTITY as i32 as u32,
                            );
                        }
                        tex_group = *psTexGroupArray[(*pp).bank as usize]
                            .offset((*pp).texGroup as isize);
                        if !tex_group.is_null() {
                            fmt = (*tex_group).fmt as GXTexFmt;
                            tex_table = &raw mut (*tex_group).texTable as *mut *mut u8;
                            width = (*tex_group).width;
                            height = (*tex_group).height;
                        } else {
                            fmt = GX_TF_I4;
                            height = 0 as u32;
                            width = 0 as u32;
                            tex_table = ::core::ptr::null_mut::<*mut u8>();
                        }
                        if !tex_table.is_null() {
                            image = *tex_table.offset((*pp).poseNum as isize)
                                as *mut ::core::ffi::c_void;
                        } else {
                            image = ::core::ptr::null_mut::<::core::ffi::c_void>();
                        }
                        if fmt as i32 == GX_TF_C4 as i32
                            || fmt as i32
                                == GX_TF_C8 as i32
                        {
                            if !tex_table.is_null() {
                                let mut palettes: *mut *mut ::core::ffi::c_void = tex_table
                                    .offset((*tex_group).num as isize) as *mut *mut u8
                                    as *mut *mut ::core::ffi::c_void;
                                if !palettes.is_null() {
                                    if (*pp).palNum as i32
                                        != 0xff as i32
                                    {
                                        tlut = *palettes.offset((*pp).palNum as isize);
                                    } else if (*pp).kind
                                        & ComTLUT as i32 as u32 == 0
                                    {
                                        tlut = *palettes.offset((*pp).poseNum as isize);
                                    } else {
                                        tlut = *palettes.offset(0 as i32 as isize);
                                    }
                                    if tlut != sp79C {
                                        tlut_obj.fmt = (*tex_group).tlutfmt as i32 as u8 as GXTlutFmt;
                                        tlut_obj.tlut_name = GX_TLUT0 as i32
                                            as u32;
                                        tlut_obj.n_entries = (if fmt as i32
                                            == GX_TF_C4 as i32
                                        {
                                            0x10 as i32
                                        } else {
                                            0x100 as i32
                                        }) as u16;
                                        GXInitTlutObj(
                                            &raw mut gx_tlut_obj,
                                            tlut,
                                            tlut_obj.fmt,
                                            tlut_obj.n_entries,
                                        );
                                        GXLoadTlut(&raw mut gx_tlut_obj, tlut_obj.tlut_name);
                                    }
                                    sp7B0 = ::core::ptr::null_mut::<::core::ffi::c_void>();
                                }
                            }
                        }
                        if sp7B0 != image && !image.is_null() {
                            sp7B0 = image;
                            match fmt as i32 {
                                8 | 9 => {
                                    GXInitTexObjCI(
                                        &raw mut sp764,
                                        image,
                                        width as u16,
                                        height as u16,
                                        fmt,
                                        wrap_s,
                                        wrap_t,
                                        0,
                                        GX_TLUT0 as i32 as u32,
                                    );
                                }
                                0 | 1 | 2 | 3 | 4 | 5 | 6 | 14 => {
                                    GXInitTexObj(
                                        &raw mut sp764,
                                        image,
                                        width as u16,
                                        height as u16,
                                        fmt,
                                        wrap_s,
                                        wrap_t,
                                        0,
                                    );
                                }
                                _ => {
                                    if 0 as i32 != 0 {} else {
                                        __assert(
                                            b"/mnt/e/Projects/Melee Workspace/worktrees/shard-961-968/rust/target/psdisp-port/psdisp.c\0"
                                                as *const u8 as *const ::core::ffi::c_char,
                                            2192 as u32,
                                            b"0\0" as *const u8 as *const ::core::ffi::c_char,
                                        );
                                    };
                                }
                            }
                            prev_tex_interp_near = ((*pp).kind
                                & TexInterpNear as i32 as u32) as s32;
                            GXInitTexObjLOD(
                                &raw mut sp764,
                                (if prev_tex_interp_near != 0 as i32 {
                                    GX_NEAR as i32
                                } else {
                                    GX_LINEAR as i32
                                }) as GXTexFilter,
                                (if (*pp).kind
                                    & TexInterpNear as i32 as u32 != 0
                                {
                                    GX_NEAR as i32
                                } else {
                                    GX_LINEAR as i32
                                }) as GXTexFilter,
                                0.0f32,
                                0.0f32,
                                0.0f32,
                                0,
                                0,
                                GX_ANISO_1,
                            );
                            GXLoadTexObj(&raw mut sp764, GX_TEXMAP0);
                        }
                        if prev_tex_interp_near as u32
                            != (*pp).kind & TexInterpNear as i32 as u32
                        {
                            prev_tex_interp_near = ((*pp).kind
                                & TexInterpNear as i32 as u32) as s32;
                            GXInitTexObjLOD(
                                &raw mut sp764,
                                (if prev_tex_interp_near != 0 as i32 {
                                    GX_NEAR as i32
                                } else {
                                    GX_LINEAR as i32
                                }) as GXTexFilter,
                                (if ((*pp).kind
                                    & TexInterpNear as i32 as u32) as s32
                                    != 0 as i32
                                {
                                    GX_NEAR as i32
                                } else {
                                    GX_LINEAR as i32
                                }) as GXTexFilter,
                                0.0f32,
                                0.0f32,
                                0.0f32,
                                0,
                                0,
                                GX_ANISO_1,
                            );
                            GXLoadTexObj(&raw mut sp764, GX_TEXMAP0);
                        }
                    }
                    if (*pp).kind & DispPoint as i32 as u32 != 0 {
                        if !(*pp).appsrt.is_null() {
                            psDispSubAPPSRTPoint(pp);
                        } else if (*pp).kind & Trail as i32 as u32 != 0
                        {
                            pp = psDispSubPointTrail(pp);
                        } else {
                            pp = psDispSubPoint(pp);
                        }
                    } else if !(*pp).appsrt.is_null() {
                        psDispSubAppSRT(pp, form);
                    } else {
                        psDispSub(pp, form);
                    }
                }
                prev_kind = (*pp).kind;
                pp = (*pp).next;
            }
        }
        sp7B4 += 1 as i32;
        if !(sp7B4 < 0x10 as i32) {
            break;
        }
    }
    if needs_setup == 0 as i32 {
        HSD_StateInvalidate(-(1 as i32));
    }
}
pub const NULL: ::core::ffi::c_long = 0 as ::core::ffi::c_long;




