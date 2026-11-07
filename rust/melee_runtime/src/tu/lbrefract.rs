//! src/melee/lb/lbrefract.c: the heat-haze / refraction effect: 32x32 IA8 distortion textures
//! generated from `LbRf.dat`, the framebuffer copy that serves as the refracted source, the
//! light-matrix setup from the current camera, the refract DObj/PObj classes, and the MObj GX
//! setup (indirect TEV stage reading the distortion texture).
//!
//! The texel callbacks stored in `lbRefract_CallbackData` all take the writer / reader signature
//! of `WriteFn` / `ReadFn` (the C calls them through one cast pointer type); narrow C parameters
//! (`u8`) are truncated inside. Data read from `LbRf.dat` (`refract_data`) is used as the archive
//! loader leaves it (native floats and pointers).
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use core::ffi::{c_char, c_void};
use core::ptr::{addr_of_mut, null_mut};
use melee_core::fp::fmadds;
use melee_types::*;

/// Callees in TUs that are not ported (dolphin gx/os, baselib class/dobj/pobj/tobj/state/memory,
/// lbarchive). Each is a settable hook; unset it panics "TODO: <name> is not ported".
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
    fn DCFlushRange(a0: *mut c_void, a1: u32);
    fn GXCopyTex(a0: *mut c_void, a1: u8);
    fn GXGetTexBufferSize(a0: u16, a1: u16, a2: u32, a3: u8, a4: u8) -> u32;
    fn GXInvalidateTexAll();
    fn GXLoadTexMtxImm(a0: *mut c_void, a1: u32, a2: u32);
    fn GXPixModeSync();
    fn GXSetAlphaCompare(a0: u32, a1: u8, a2: u32, a3: u32, a4: u8) = crate::tu::lbgx::ext::GXSetAlphaCompare(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _);
    fn GXSetAlphaUpdate(a0: u8) = crate::tu::lbgx::ext::GXSetAlphaUpdate(a0 as _);
    fn GXSetBlendMode(a0: u32, a1: u32, a2: u32, a3: u32) = crate::tu::lbgx::ext::GXSetBlendMode(a0 as _, a1 as _, a2 as _, a3 as _);
    fn GXSetColorUpdate(a0: u8) = crate::tu::lbgx::ext::GXSetColorUpdate(a0 as _);
    fn GXSetDither(a0: u8);
    fn GXSetIndTexCoordScale(a0: u32, a1: u32, a2: u32);
    fn GXSetIndTexMtx(a0: u32, a1: *mut c_void, a2: i8);
    fn GXSetIndTexOrder(a0: u32, a1: u32, a2: u32);
    fn GXSetNumChans(a0: u8) = crate::tu::lbgx::ext::GXSetNumChans(a0 as _);
    fn GXSetNumIndStages(a0: u8);
    fn GXSetNumTevStages(a0: u8) = crate::tu::lbgx::ext::GXSetNumTevStages(a0 as _);
    fn GXSetNumTexGens(a0: u8) = crate::tu::lbgx::ext::GXSetNumTexGens(a0 as _);
    fn GXSetTevAlphaIn(a0: u32, a1: u32, a2: u32, a3: u32, a4: u32);
    fn GXSetTevAlphaOp(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32);
    fn GXSetTevColorIn(a0: u32, a1: u32, a2: u32, a3: u32, a4: u32);
    fn GXSetTevColorOp(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32);
    fn GXSetTevDirect(a0: u32);
    fn GXSetTevIndirect(a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, a7: u8, a8: u8, a9: u32);
    fn GXSetTevOrder(a0: u32, a1: u32, a2: u32, a3: u32) = crate::tu::lbgx::ext::GXSetTevOrder(a0 as _, a1 as _, a2 as _, a3 as _);
    fn GXSetTexCoordGen2(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32);
    fn GXSetTexCopyDst(a0: u16, a1: u16, a2: u32, a3: u8);
    fn GXSetTexCopySrc(a0: u16, a1: u16, a2: u16, a3: u16);
    fn GXSetZCompLoc(a0: u8) = crate::tu::lbgx::ext::GXSetZCompLoc(a0 as _);
    fn GXSetZMode(a0: u8, a1: u32, a2: u8) = crate::tu::lbgx::ext::GXSetZMode(a0 as _, a1 as _, a2 as _);
    fn HSD_CObjGetProjectionType(a0: *mut HSD_CObj) -> i32 = (|c| unsafe { crate::tu::cobj::HSD_CObjGetProjectionType(c) })(a0);
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = crate::tu::lbheap::ext::HSD_MemAlloc(a0 as _);
    fn HSD_PObjGetFlags(a0: *mut HSD_PObj) -> u32 = crate::tu::pobj::HSD_PObjGetFlags(a0);
    fn HSD_StateInvalidate(a0: i32);
    fn HSD_TObjLoadDesc(a0: *mut _HSD_TObjDesc) -> *mut HSD_TObj = (|d| unsafe { crate::tu::tobj::HSD_TObjLoadDesc(d) })(a0);
    fn HSD_TObjSetup(a0: *mut HSD_TObj);
    fn MTXLightFrustum(a0: *mut c_void, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32, a8: f32, a9: f32);
    fn MTXLightOrtho(a0: *mut c_void, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32, a8: f32);
    fn MTXLightPerspective(a0: *mut c_void, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32);
    fn hsdInitClassInfo(a0: *mut _HSD_ClassInfo, a1: *mut _HSD_ClassInfo, a2: *mut i8, a3: *mut i8, a4: i32, a5: i32) = (|info, parent, lib, name, size_info, size_obj| unsafe { crate::tu::class::hsdInitClassInfo(info as _, parent as _, lib as _, name as _, size_info, size_obj) })(a0, a1, a2, a3, a4, a5);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut hsdDObj: HSD_DObjInfo = unsafe { core::mem::zeroed() };
    pub static mut hsdPObj: HSD_PObjInfo = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `lbArchive_LoadSymbols(file, out, sym)` (variadic in the C): settable hook, no port.
pub static mut H_lbArchive_LoadSymbols: Option<fn(*const c_char, *mut c_void, *const c_char)> = None;
pub fn lbArchive_LoadSymbols(file: *const c_char, out: *mut c_void, sym: *const c_char) {
    melee_core::ct!("lbArchive_LoadSymbols");
    match unsafe { H_lbArchive_LoadSymbols } { Some(f) => f(file, out, sym), None => melee_core::unported!("TODO: lbArchive_LoadSymbols is not ported") }
}

const FILE: &str = "src/melee/lb/lbrefract.c";

/// `lbl_804336D0`.
#[repr(C)]
pub struct RefractState {
    pub refractionUserCount: i32,
    pub image_ptr: *mut c_void,
    pub imagedesc: *mut HSD_ImageDesc,
    pub tobj_list: *mut *mut HSD_TObj,
    pub texture_mtx: Mtx,
}
pub static mut lbl_804336D0: RefractState =
    RefractState { refractionUserCount: 0, image_ptr: null_mut(), imagedesc: null_mut(), tobj_list: null_mut(), texture_mtx: [[0.0; 4]; 3] };

/// `refract_data`: the `lbRefData` symbol of LbRf.dat.
#[repr(C)]
pub struct RefractData {
    pub x0: u8,
    pub x4: *mut f32,
}
pub static mut refract_data: *mut RefractData = null_mut();

/// Texel writer: (cb, row, col, arg3, arg4, arg5, arg6); the C types of the arguments differ per
/// format, the extra ones are ignored.
pub type WriteFn = unsafe extern "C" fn(*mut lbRefract_CallbackData, i32, i32, u32, u32, u32, u32);
/// Texel reader: (cb, row, col, out3, out4, out5, out6); outputs may be null.
pub type ReadFn = unsafe extern "C" fn(*mut lbRefract_CallbackData, u32, u32, *mut i32, *mut i32, *mut i32, *mut i32);

/// `lbRefract_WriteTexCoord` (static inline): packs the (y, x) displacement into two bytes.
unsafe fn lbRefract_WriteTexCoord(cb: *mut lbRefract_CallbackData, row: i32, col: u32, y: f32, x: f32, param0: f32) {
    use crate::cabi::f2u32;
    let y_tex = f2u32(fmadds(127.0, y * param0, 128.0) as f64);
    let x_tex = f2u32(fmadds(127.0, x * param0, 128.0) as f64);
    let f: WriteFn = core::mem::transmute((*cb).callback0);
    f(cb, row, col as i32, 0, 0, y_tex, x_tex);
}

/// `my_fmodf` (static inline): `a - b * (s64)(a / b)` (one fused site), or `a` when |b| > |a|.
fn my_fmodf(a: f32, b: f32) -> f32 {
    let fa = a.abs();
    let fb = b.abs();
    if fb > fa {
        return a;
    }
    let quotient = melee_core::ppc::fcvt_i64((a / b));
    fmadds(-b, quotient as f32, a)
}

/// `lbRefract_80021CE8(cb, arg1)`: fills the 32x32 distortion texture `arg1` from LbRf.dat's
/// radial parameters (modulus, scale) and flushes it.
pub unsafe fn lbRefract_80021CE8(cb: *mut lbRefract_CallbackData, arg1: i32) {
    melee_core::ct!("lbRefract_80021CE8");
    use crate::gekko::sqrtf;
    let param_idx = (arg1 * 2) as isize;
    let x_step = 2.0f32 / ((*cb).width.wrapping_sub(1) as f32);
    let y_step = 2.0f32 / ((*cb).height.wrapping_sub(1) as f32);
    let mut y = -1.0f32;
    let x4 = (*refract_data).x4;

    let mut row = 0i32;
    while (row as u32) < (*cb).height {
        let mut x = -1.0f32;
        let y_sq = y * y;
        let mut col = 0i32;
        while (col as u32) < (*cb).width {
            let mut dist = sqrtf(fmadds(x, x, y_sq));
            if dist > 1.0 {
                dist = 1.0;
            }
            let mut param0 = *x4.offset(param_idx);
            if param0 != 0.0 {
                param0 = dist * my_fmodf(dist, param0);
            } else {
                param0 = dist;
            }
            param0 *= *x4.offset(param_idx + 1);
            if param0 > 1.0 {
                param0 = 1.0;
            }
            lbRefract_WriteTexCoord(cb, col, row as u32, y, x, param0);
            x += x_step;
            col += 1;
        }
        y += y_step;
        row += 1;
    }
    ext::DCFlushRange((*cb).buffer, (*cb).buffer_size as u32);
}

/// `(u8*) data->buffer + c * data->row_stride + off` in the C's u32 arithmetic.
#[inline]
unsafe fn tile_base(data: *const lbRefract_CallbackData, c: u32, off: u32) -> *mut u8 {
    ((*data).buffer as *mut u8).add(c.wrapping_mul((*data).row_stride).wrapping_add(off) as usize)
}

/// `lbRefract_WriteTexCoordIA4` (an IA8 texel writer): alpha then intensity.
pub unsafe extern "C" fn lbRefract_WriteTexCoordIA4(data: *mut lbRefract_CallbackData, row: u32, col: u32, _arg3: u32, _arg4: u32, intensity: u32, alpha: u32) {
    melee_core::ct!("lbRefract_WriteTexCoordIA4");
    let (row, col) = (row as u32, col as u32);
    let base = tile_base(data, col >> 2, (row >> 2).wrapping_mul(32));
    let i = ((row & 3) + (col & 3) * 4) as usize;
    *base.add(2 * i) = alpha as u8;
    *base.add(2 * i + 1) = intensity as u8;
}

/// `fn_80021F70`: writes one RGB565 texel.
unsafe extern "C" fn fn_80021F70(data: *mut lbRefract_CallbackData, row: i32, col: i32, r: u32, g: u32, b: u32, _unused: u32) {
    let (row, col) = (row as u32, col as u32);
    let (r, g) = (r as u8, g as u8);
    let base = tile_base(data, col >> 2, (row >> 2).wrapping_mul(32)) as *mut u16;
    let offset = (row & 3) + (col & 3) * 4;
    let v = (b / 8) | (((g as i32 * 8) & 0x7E0 & !0xF800) | (((r as i32) << 8) & 0xF800)) as u32;
    base.add(offset as usize).write_unaligned(v as u16);
}

/// `fn_80021FB4`: writes one RGBA8 texel (AR and GB halves 32 bytes apart).
unsafe extern "C" fn fn_80021FB4(data: *mut lbRefract_CallbackData, row: i32, col: i32, arg6: u32, arg7: u32, arg8: u32, arg9: u32) {
    let (row, col) = (row as u32, col as u32);
    let base = tile_base(data, col >> 2, (row >> 2).wrapping_mul(64));
    let i = ((row & 3) + (col & 3) * 4) as usize;
    *base.add(2 * i) = arg9 as u8;
    *base.add(2 * i + 1) = arg6 as u8;
    *base.add(2 * (i + 16)) = arg7 as u8;
    *base.add(2 * (i + 16) + 1) = arg8 as u8;
}

/// `fn_80021FF8`: reads one IA8 texel; the first two outputs get 0xFF.
unsafe extern "C" fn fn_80021FF8(data: *mut lbRefract_CallbackData, row: u32, col: u32, arg3: *mut i32, arg4: *mut i32, arg5: *mut i32, arg6: *mut i32) {
    let base = tile_base(data, col >> 2, (row >> 2).wrapping_mul(32));
    let offset = (((row & 3) + (col & 3) * 4) * 2) as usize;
    if !arg3.is_null() {
        *arg3 = 0xFF;
    }
    if !arg4.is_null() {
        *arg4 = 0xFF;
    }
    if !arg5.is_null() {
        *arg5 = *base.add(offset + 1) as i32;
    }
    if !arg6.is_null() {
        *arg6 = *base.add(offset) as i32;
    }
}

/// `fn_8002206C`: reads one RGB565 texel back as 3/2/3-bit presence flags.
unsafe extern "C" fn fn_8002206C(data: *mut lbRefract_CallbackData, row: u32, col: u32, arg3: *mut i32, arg4: *mut i32, arg5: *mut i32, arg6: *mut i32) {
    let r = row & !3;
    let c = col & !3;
    let base = tile_base(data, c / 4, r.wrapping_mul(8)) as *mut u16;
    let idx = (row & 3) + (col & 3) * 4;
    let pixel = base.add(idx as usize).read_unaligned() as u32;
    if !arg6.is_null() {
        *arg6 = 0xFF;
    }
    if !arg3.is_null() {
        *arg3 = if ((pixel >> 8) & 0xF8) | (pixel & 0x8000) != 0 { 7 } else { 0 };
    }
    if !arg4.is_null() {
        *arg4 = if ((pixel >> 3) & 0xFC) | (pixel & 0x400) != 0 { 3 } else { 0 };
    }
    if !arg5.is_null() {
        *arg5 = if pixel & 0xF8 != 0 { 7 } else { 0 };
    }
}

/// `lbRefract_ReadTexCoordRGBA8`: reads one RGBA8 texel (outputs may be null).
unsafe extern "C" fn lbRefract_ReadTexCoordRGBA8(data: *mut lbRefract_CallbackData, row: u32, col: u32, out_r: *mut i32, out_g: *mut i32, out_b: *mut i32, out_a: *mut i32) {
    let base = tile_base(data, col >> 2, (row >> 2).wrapping_mul(64));
    let offset = (((row & 3) + (col & 3) * 4) * 2) as usize;
    if !out_a.is_null() {
        *out_a = *base.add(offset) as i32;
    }
    if !out_r.is_null() {
        *out_r = *base.add(offset + 1) as i32;
    }
    if !out_g.is_null() {
        *out_g = *base.add(offset + 0x20) as i32;
    }
    if !out_b.is_null() {
        *out_b = *base.add(offset + 0x21) as i32;
    }
}

/// `lbRefract_8002219C`: initialises `data` for a texture buffer of format IA8 / RGB565 / RGBA8
/// (picks the texel callbacks and the row stride); 0 on success, -1 for any other format.
unsafe fn lbRefract_8002219C(data: *mut lbRefract_CallbackData, buffer: *mut c_void, format: i32, width: u32, height: u32) -> i32 {
    (*data).buffer = buffer;
    (*data).format = format;
    (*data).width = width;
    (*data).height = height;
    (*data).buffer_size = ext::GXGetTexBufferSize(width as u16, height as u16, format as u32, 0, 0) as i32;
    let ia8 = GX_TF_IA8 as i32;
    let rgb565 = GX_TF_RGB565 as i32;
    let rgba8 = GX_TF_RGBA8 as i32;
    if format == ia8 {
        (*data).callback0 = lbRefract_WriteTexCoordIA4 as unsafe extern "C" fn(*mut lbRefract_CallbackData, u32, u32, u32, u32, u32, u32) as usize as *mut c_void;
        (*data).callback1 = fn_80021FF8 as ReadFn as *mut c_void;
        (*data).row_stride = width.wrapping_mul(8) & 0xFFFF_FFE0;
    } else if format == rgb565 {
        (*data).callback0 = fn_80021F70 as WriteFn as *mut c_void;
        (*data).callback1 = fn_8002206C as ReadFn as *mut c_void;
        (*data).row_stride = width.wrapping_mul(8) & 0xFFFF_FFE0;
    } else if format == rgba8 {
        (*data).callback0 = fn_80021FB4 as WriteFn as *mut c_void;
        (*data).callback1 = lbRefract_ReadTexCoordRGBA8 as ReadFn as *mut c_void;
        (*data).row_stride = width.wrapping_mul(16) & 0xFFFF_FFC0;
    } else {
        return -1;
    }
    0
}

/// `texture_mtx`.
pub static mut texture_mtx: Mtx = [[0.5, 0.0, 0.0, 0.5], [0.0, -0.5, 0.0, 0.5], [0.0, 0.0, 0.0, 1.0]];
/// `texture_offset`.
pub static mut texture_offset: [[f32; 3]; 2] = [[-0.5, 0.0, 0.0], [0.0, -0.5, 0.0]];

pub static mut imagedesc0: HSD_ImageDesc = HSD_ImageDesc { image_ptr: null_mut(), width: 0, height: 0, format: 4, mipmap: 0, minLOD: 0.0, maxLOD: 0.0 };
pub static mut loddesc0: HSD_TexLODDesc = HSD_TexLODDesc { minFilt: 1, LODBias: 0.0, bias_clamp: 1, edgeLODEnable: 1, max_anisotropy: 0 };
pub static mut imagedesc1: HSD_ImageDesc = HSD_ImageDesc { image_ptr: null_mut(), width: 0, height: 0, format: 3, mipmap: 0, minLOD: 0.0, maxLOD: 0.0 };
pub static mut loddesc1: HSD_TexLODDesc = HSD_TexLODDesc { minFilt: 1, LODBias: 0.0, bias_clamp: 1, edgeLODEnable: 1, max_anisotropy: 0 };

const fn tobj_desc(next: *mut HSD_TObjDesc, id: u32, src: u32, blend_flags: u32, image: *mut HSD_ImageDesc, lod: *mut HSD_TexLODDesc) -> HSD_TObjDesc {
    let mut d: HSD_TObjDesc = unsafe { core::mem::zeroed() };
    d.next = next;
    d.id = id;
    d.src = src;
    d.scale = Vec { x: 1.0, y: 1.0, z: 1.0 };
    d.repeat_s = 1;
    d.repeat_t = 1;
    d.blend_flags = blend_flags;
    d.blending = 1.0;
    d.magFilt = 1;
    d.imagedesc = image;
    d.lod = lod;
    d
}

pub static mut tobjdesc0: HSD_TObjDesc = tobj_desc(null_mut(), 0, 0, 0x83, unsafe { addr_of_mut!(imagedesc0) }, unsafe { addr_of_mut!(loddesc0) });
pub static mut tobjdesc1: HSD_TObjDesc = tobj_desc(unsafe { addr_of_mut!(tobjdesc0) }, 1, 1, 0x81, unsafe { addr_of_mut!(imagedesc1) }, unsafe { addr_of_mut!(loddesc1) });

/// `lbRefract_800222A4`: loads LbRf.dat, allocates the 320x240 RGB565 source texture and, per
/// refraction table entry, a 32x32 IA8 distortion texture with its TObj.
pub unsafe fn lbRefract_800222A4() {
    melee_core::ct!("lbRefract_800222A4");
    const IMAGE_WIDTH: i32 = 320;
    const IMAGE_HEIGHT: i32 = 240;
    let mut cb = lbRefract_CallbackData::zeroed();

    lbl_804336D0.refractionUserCount = 0;
    lbArchive_LoadSymbols(b"LbRf.dat\0".as_ptr() as *const c_char, addr_of_mut!(refract_data) as *mut c_void, b"lbRefData\0".as_ptr() as *const c_char);
    {
        let buf_size = ext::GXGetTexBufferSize(IMAGE_WIDTH as u16, IMAGE_HEIGHT as u16, GX_TF_RGB565 as u32, 0, 0);
        lbl_804336D0.image_ptr = ext::HSD_MemAlloc(buf_size as i32);
        core::ptr::write_bytes(lbl_804336D0.image_ptr as *mut u8, 0, buf_size as usize);
    }
    let n = (*refract_data).x0 as usize;
    lbl_804336D0.tobj_list = ext::HSD_MemAlloc((n * core::mem::size_of::<*mut HSD_TObj>()) as i32) as *mut *mut HSD_TObj;
    lbl_804336D0.imagedesc = ext::HSD_MemAlloc((n * core::mem::size_of::<HSD_ImageDesc>()) as i32) as *mut HSD_ImageDesc;

    let mut i = 0usize;
    while i < n {
        let buf = ext::HSD_MemAlloc(ext::GXGetTexBufferSize(32, 32, GX_TF_IA8 as u32, 0, 0) as i32);
        lbRefract_8002219C(&mut cb, buf, GX_TF_IA8 as i32, 32, 32);
        lbRefract_80021CE8(&mut cb, i as i32);

        *lbl_804336D0.imagedesc.add(i) = imagedesc0;
        tobjdesc1.imagedesc = lbl_804336D0.imagedesc.add(i);
        *lbl_804336D0.tobj_list.add(i) = ext::HSD_TObjLoadDesc(addr_of_mut!(tobjdesc1));

        imagedesc0.image_ptr = lbl_804336D0.image_ptr;
        imagedesc0.format = GX_TF_RGB565;
        imagedesc0.width = IMAGE_WIDTH as u16;
        imagedesc0.height = IMAGE_HEIGHT as u16;

        let d = &mut *lbl_804336D0.imagedesc.add(i);
        d.image_ptr = buf;
        d.format = GX_TF_IA8;
        d.width = 32;
        d.height = 32;
        i += 1;
    }
}

/// `lbRefract_8002247C(cobj)`: derives the light-projection texture matrix from the camera's
/// projection (perspective / frustum / ortho) while refraction has users.
pub unsafe fn lbRefract_8002247C(cobj: *mut HSD_CObj) {
    melee_core::ct!("lbRefract_8002247C");
    use crate::mtx::{light_frustum, light_ortho, light_perspective};
    if lbl_804336D0.refractionUserCount == 0 {
        return;
    }
    let p = &(*cobj).projection_param;
    let t = ext::HSD_CObjGetProjectionType(cobj);
    lbl_804336D0.texture_mtx = if t == PROJ_PERSPECTIVE {
        light_perspective(p.perspective.fov, p.perspective.aspect, 0.5, -0.5, 0.5, 0.5)
    } else if t == PROJ_FRUSTUM {
        light_frustum(p.frustum.top, p.frustum.bottom, p.frustum.left, p.frustum.right, (*cobj).near, 0.5, -0.5, 0.5, 0.5)
    } else {
        light_ortho(p.ortho.top, p.ortho.bottom, p.ortho.left, p.ortho.right, 0.5, -0.5, 0.5, 0.5)
    };
}

/// `lbRefract_80022560`: copies the 640x480 framebuffer (downscaled to 320x240 RGB565) into the
/// refraction source texture while refraction has users.
pub unsafe fn lbRefract_80022560() {
    melee_core::ct!("lbRefract_80022560");
    if lbl_804336D0.refractionUserCount != 0 {
        ext::GXSetTexCopySrc(0, 0, 0x280, 0x1E0);
        ext::GXSetTexCopyDst(0x140, 0xF0, 4, 1);
        ext::GXCopyTex(lbl_804336D0.image_ptr, 0);
        ext::GXPixModeSync();
        ext::GXInvalidateTexAll();
    }
}

/// `lbRefract_800225D4`: resets the TEV and indirect texture stages.
pub fn lbRefract_800225D4() {
    melee_core::ct!("lbRefract_800225D4");
    ext::GXSetTevDirect(0);
    ext::GXSetNumIndStages(0);
    ext::HSD_StateInvalidate(-1);
}

/// `lbRefract_DObjDispReset`: the refract DObj's `disp`: the base display, then a TEV reset.
unsafe extern "C" fn lbRefract_DObjDispReset(dobj: *mut HSD_DObj, vmtx: *mut [f32; 4], pmtx: *mut [f32; 4], rendermode: u32) {
    ((*dobj_info_ptr()).disp.expect("hsdDObj.disp"))(dobj, vmtx, pmtx, rendermode);
    ext::GXSetTevDirect(0);
    ext::GXSetNumIndStages(0);
    ext::HSD_StateInvalidate(HSD_STATE_ALL);
}

/// `dobj_info`: class info of "refract_dobj" (`info_init` = `fn_80022650`).
pub static mut dobj_info: HSD_DObjInfo = {
    let mut v: HSD_DObjInfo = unsafe { core::mem::zeroed() };
    v.parent.head.info_init = Some(fn_80022650);
    v
};
/// `pobj_info`: class info of "refract_pobj" (`info_init` = `fn_80022940`).
pub static mut pobj_info: HSD_PObjInfo = {
    let mut v: HSD_PObjInfo = unsafe { core::mem::zeroed() };
    v.parent.head.info_init = Some(fn_80022940);
    v
};

/// Parent class infos for `hsdInitClassInfo` (`hsdDObj`, `hsdPObj`); set by the owner of those
/// classes.
/// Tests that own their copy of other TUs' data set this; the game uses the owning TU's statics.
pub static mut use_ext_data: bool = false;
unsafe fn dobj_info_ptr() -> *mut HSD_DObjInfo {
    if use_ext_data { addr_of_mut!(ext::hsdDObj) } else { addr_of_mut!(crate::tu::dobj::hsd::hsdDObj) }
}
unsafe fn pobj_info_ptr() -> *mut HSD_PObjInfo {
    if use_ext_data { addr_of_mut!(ext::hsdPObj) } else { addr_of_mut!(crate::tu::pobj::hsdPObj) }
}

/// `fn_80022650`: registers the refract DObj class under `hsdDObj`.
pub unsafe extern "C" fn fn_80022650() {
    melee_core::ct!("fn_80022650");
    ext::hsdInitClassInfo(
        addr_of_mut!(dobj_info) as *mut _HSD_ClassInfo,
        dobj_info_ptr().cast(),
        b"refract_class_library\0".as_ptr() as *mut c_char,
        b"refract_dobj\0".as_ptr() as *mut c_char,
        core::mem::size_of::<HSD_DObjInfo>() as i32,
        core::mem::size_of::<HSD_DObj>() as i32,
    );
    dobj_info.disp = Some(lbRefract_DObjDispReset);
}

/// `lbRefract_PObjLoad`: the refract PObj's `load`: after the base load, for PObjs with flag
/// 0x2000 copies each vertex's PNMTXIDX byte into its last TEXnMTXIDX slot throughout the display
/// list. Returns the base load's error code, else 0.
pub unsafe extern "C" fn lbRefract_PObjLoad(pobj: *mut HSD_PObj, desc: *mut HSD_PObjDesc) -> i32 {
    melee_core::ct!("lbRefract_PObjLoad");
    let ret = ((*pobj_info_ptr()).load.expect("hsdPObj.load"))(pobj, desc);
    if ret != 0 {
        return ret;
    }
    if ext::HSD_PObjGetFlags(pobj) & 0x2000 == 0 {
        return 0;
    }

    let mut verts = (*pobj).verts;
    let mut stride = 0i32;
    let mut pnmtx_offset = -1i32;
    let mut last_offset = -1i32;

    while !verts.is_null() && (*verts).attr != GX_VA_NULL {
        let attr = (*verts).attr;
        if (GX_VA_TEX0MTXIDX..=GX_VA_TEX7MTXIDX).contains(&attr) {
            last_offset = stride;
            stride += 1;
        } else if attr == GX_VA_PNMTXIDX {
            pnmtx_offset = stride;
            stride += 1;
        } else if (GX_VA_TEX0..=GX_VA_TEX7).contains(&attr) || attr == GX_VA_POS || attr == GX_VA_NRM {
            if (*verts).attr_type == GX_INDEX16 {
                stride += 2;
            } else {
                stride += 1;
            }
        } else if attr == GX_VA_CLR0 || attr == GX_VA_CLR1 {
            let at = (*verts).attr_type;
            if at == GX_INDEX16 {
                stride += 2;
            } else if at == GX_INDEX8 {
                stride += 1;
            } else {
                let ct = (*verts).comp_type;
                if ct == GX_RGB565 || ct == GX_RGBA4 {
                    stride += 2;
                } else if ct == GX_RGB8 || ct == GX_RGBA6 {
                    stride += 3;
                } else if ct == GX_RGBX8 {
                    stride += 4;
                }
            }
        }
        verts = verts.add(1);
    }

    if pnmtx_offset == -1 || last_offset < 2 {
        return 0;
    }

    let display = (*pobj).display;
    let mut offset = 0i32;
    let total_bytes = ((*pobj).n_display as i32) << 5;
    while offset < total_bytes {
        let b = *display.offset(offset as isize);
        offset += 1;
        if b & 0xF8 == 0 {
            break;
        }
        let ptr = display.offset(offset as isize);
        let hi = *ptr as i32;
        let mut count = hi << 8;
        count |= *ptr.add(1) as i32;
        offset += 2;

        let mut copied = 0;
        while copied < count {
            *display.offset((offset + last_offset) as isize) = *display.offset((offset + pnmtx_offset) as isize);
            offset += stride;
            copied += 1;
        }
    }

    ext::DCFlushRange(display as *mut c_void, total_bytes as u32);
    0
}

/// `fn_80022940`: registers the refract PObj class under `hsdPObj`.
pub unsafe extern "C" fn fn_80022940() {
    melee_core::ct!("fn_80022940");
    ext::hsdInitClassInfo(
        addr_of_mut!(pobj_info) as *mut _HSD_ClassInfo,
        pobj_info_ptr().cast(),
        b"refract_class_library\0".as_ptr() as *mut c_char,
        b"refract_pobj\0".as_ptr() as *mut c_char,
        core::mem::size_of::<HSD_PObjInfo>() as i32,
        core::mem::size_of::<HSD_PObj>() as i32,
    );
    pobj_info.load = Some(lbRefract_PObjLoad);
}

/// `lbRefract_80022998(mobj, rendermode, arg2)`: sets up TObj `arg2` and the GX state that
/// samples the refraction source through the distortion texture (indirect TEV stage).
pub unsafe fn lbRefract_80022998(_mobj: *mut HSD_MObj, rendermode: u32, arg2: i32) {
    melee_core::ct!("lbRefract_80022998");
    ext::HSD_TObjSetup(*lbl_804336D0.tobj_list.offset(arg2 as isize));

    ext::GXSetNumTexGens(2);
    ext::GXSetTexCoordGen2(GX_TEXCOORD0 as _, GX_TG_MTX3x4 as _, GX_TG_NRM as _, GX_TEXMTX0 as _, 1, GX_PTTEXMTX0 as _);
    ext::GXSetTexCoordGen2(GX_TEXCOORD1 as _, GX_TG_MTX3x4 as _, GX_TG_POS as _, 0, 0, GX_PTTEXMTX1 as _);

    ext::GXLoadTexMtxImm(addr_of_mut!(texture_mtx) as *mut c_void, GX_PTTEXMTX0 as _, GX_MTX3x4 as _);
    ext::GXLoadTexMtxImm(addr_of_mut!(lbl_804336D0.texture_mtx) as *mut c_void, GX_PTTEXMTX1 as _, GX_MTX3x4 as _);

    ext::GXSetNumChans(0);
    ext::GXSetNumTevStages(1);
    let write_z0 = GX_ITF_8 as i32;
    ext::GXSetTevOrder(GX_TEVSTAGE0 as _, GX_TEXCOORD1 as _, GX_TEXMAP1 as _, GX_COLOR_NULL as _);
    ext::GXSetTevColorIn(GX_TEVSTAGE0 as _, GX_CC_ZERO as _, GX_CC_ZERO as _, GX_CC_ZERO as _, GX_CC_TEXC as _);
    ext::GXSetTevAlphaIn(GX_TEVSTAGE0 as _, GX_CA_ZERO as _, GX_CA_ZERO as _, GX_CA_ZERO as _, GX_CA_TEXA as _);
    ext::GXSetTevColorOp(GX_TEVSTAGE0 as _, GX_TEV_ADD as _, GX_TB_ZERO as _, GX_CS_SCALE_1 as _, 0, GX_TEVPREV as _);
    ext::GXSetTevAlphaOp(GX_TEVSTAGE0 as _, GX_TEV_ADD as _, GX_TB_ZERO as _, GX_CS_SCALE_1 as _, 0, GX_TEVPREV as _);
    ext::GXSetNumIndStages(1);
    ext::GXSetIndTexOrder(GX_INDTEXSTAGE0 as _, GX_TEXCOORD0 as _, GX_TEXMAP0 as _);
    ext::GXSetIndTexCoordScale(GX_INDTEXSTAGE0 as _, GX_ITS_1 as _, GX_ITS_1 as _);
    ext::GXSetIndTexMtx(GX_ITM_0 as _, addr_of_mut!(texture_offset) as *mut c_void, 1);

    ext::GXSetTevIndirect(GX_TEVSTAGE0 as _, GX_INDTEXSTAGE0 as _, write_z0 as _, GX_ITB_ST as _, GX_ITM_0 as _, GX_ITW_OFF as _, GX_ITW_OFF as _, 0, 0, GX_ITBA_OFF as _);

    ext::GXSetColorUpdate(1);
    ext::GXSetAlphaUpdate(0);

    let write_z: u8 = if rendermode & (RENDER_NO_ZUPDATE as u32) != 0 { 0 } else { 1 };
    let compare = if rendermode & (RENDER_ZMODE_ALWAYS as u32) != 0 { GX_ALWAYS } else { GX_LEQUAL };

    ext::GXSetZMode(1, compare as _, write_z);
    ext::GXSetZCompLoc(1);
    ext::GXSetBlendMode(GX_BM_NONE as _, GX_BL_ONE as _, GX_BL_ZERO as _, GX_LO_SET as _);
    ext::GXSetAlphaCompare(GX_ALWAYS as _, 0, GX_AOP_OR as _, GX_ALWAYS as _, 0);
    ext::GXSetDither(0);
}

/// `lbRefract_80022BB8`: one more refraction user.
pub fn lbRefract_80022BB8() {
    melee_core::ct!("lbRefract_80022BB8");
    unsafe { lbl_804336D0.refractionUserCount += 1 };
}

/// `lbRefSetUnuse`: one refraction user less; asserts when the count goes negative.
pub fn lbRefSetUnuse() {
    melee_core::ct!("lbRefSetUnuse");
    unsafe {
        lbl_804336D0.refractionUserCount -= 1;
        if lbl_804336D0.refractionUserCount < 0 {
            crate::tu::lbext::OSReport("lbRefSetUnuse error!\n", &[]);
            crate::tu::lbext::__assert(FILE, 0x31c, "0");
        }
    }
}
