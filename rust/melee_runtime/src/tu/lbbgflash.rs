//! Port of src/melee/lb/lbbgflash.c (full-screen colour flashes and the wipe strips of the
//! background-flash GObjs). Verified against the C (whole TU, shipping contraction) by
//! tests/tu_lbbgflash.rs (`python rust/melee_runtime/gen/mtu/run_tu.py lbbgflash`).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

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
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32);
    fn GXBegin(a0: u32, a1: u32, a2: u16);
    fn GXColor4u8(a0: u8, a1: u8, a2: u8, a3: u8);
    fn GXPosition2f32(a0: f32, a1: f32);
    fn HSD_CObjGetAspect(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj = crate::tu::cobj::HSD_CObjGetCurrent();
    fn HSD_CObjLoadDesc(a0: *mut HSD_CObjDesc) -> *mut HSD_CObj = crate::tu::cobj::HSD_CObjLoadDesc(a0);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32);
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn hsd_80391A04(a0: f32, a1: f32, a2: i32) = crate::tu::hsd_3915::native_80391A04(a0, a1, a2);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_803910D8(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::gobj_native::HSD_GObj_CameraKind;
}
// END GENERATED EXT

/// `BgFlashState`: `u8 active : 1; u8 mode : 7;` in GameCube (MSB-first) bit order.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BgFlashState {
    pub bits: u8,
}
impl BgFlashState {
    pub fn active(&self) -> u8 {
        self.bits >> 7
    }
    pub fn set_active(&mut self, v: u8) {
        self.bits = (self.bits & 0x7F) | ((v & 1) << 7);
    }
    pub fn mode(&self) -> u8 {
        self.bits & 0x7F
    }
    pub fn set_mode(&mut self, v: u8) {
        self.bits = (self.bits & 0x80) | (v & 0x7F);
    }
}
melee_core::gc_layout!(BgFlashState, "BgFlashState", size 0x1, align 1, [
    active @ 0x0: 1 "u8" bits 0 + 1,
    mode @ 0x0: 1 "u8" bits 1 + 7,
]);

/// `BgFlashData` (0x48 bytes on the console).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BgFlashData {
    pub state: BgFlashState,
    pub pad1: [u8; 3],
    pub x4: GXColor,
    pub x8: GXColor,
    pub xC: GXColor,
    pub x10: [f32; 4],
    pub x20: [f32; 4],
    pub x30: u8,
    pub x31: u8,
    pub x32: u8,
    pub x33: u8,
    pub x34: i32,
    pub x38: i32,
    pub x3C: i32,
    pub x40: *mut HSD_GObj,
    pub x44: *mut HSD_GObj,
}
melee_core::gc_layout!(BgFlashData, "BgFlashData", size 0x48, align 4, [
    state @ 0x0: 1 "BgFlashState",
    pad1 @ 0x1: 3 "u8[3]",
    x4 @ 0x4: 4 "GXColor",
    x8 @ 0x8: 4 "GXColor",
    xC @ 0xC: 4 "GXColor",
    x10 @ 0x10: 16 "f32[4]",
    x20 @ 0x20: 16 "f32[4]",
    x30 @ 0x30: 1 "u8",
    x31 @ 0x31: 1 "u8",
    x32 @ 0x32: 1 "u8",
    x33 @ 0x33: 1 "u8",
    x34 @ 0x34: 4 "s32",
    x38 @ 0x38: 4 "s32",
    x3C @ 0x3C: 4 "s32",
    x40 @ 0x40: 4 "HSD_GObj*",
    x44 @ 0x44: 4 "HSD_GObj*",
]);

/// 433658
pub static mut lbl_80433658: BgFlashData = unsafe { core::mem::zeroed() };

static mut lbl_804D3840: GXColor = GXColor { r: 0, g: 0, b: 0, a: 255 };
static mut lbl_804D3844: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };
static mut lbl_804D3848: GXColor = GXColor { r: 255, g: 255, b: 255, a: 255 };
static mut lbl_804D384C: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };

pub static mut lbl_803BB000: HSD_WObjDesc =
    HSD_WObjDesc { class_name: null_mut(), pos: Vec { x: 320.0, y: -240.0, z: 415.69220 }, robjdesc: null_mut() };
pub static mut lbl_803BB014: HSD_WObjDesc =
    HSD_WObjDesc { class_name: null_mut(), pos: Vec { x: 320.0, y: -240.0, z: 0.0 }, robjdesc: null_mut() };

/// 3BB028
pub static mut lbl_803BB028: HSD_CameraDescPerspective = HSD_CameraDescPerspective {
    class_name: null_mut(),
    flags: 0,
    projection_type: PROJ_PERSPECTIVE as u16,
    viewport: _HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 480 },
    scissor: _Scissor { left: 0, right: 640, top: 0, bottom: 480 },
    eyepos: unsafe { &raw mut lbl_803BB000 },
    interest: unsafe { &raw mut lbl_803BB014 },
    roll: 0.0,
    up_vector: null_mut(),
    nnear: 0.1,
    ffar: 30000.0,
    fov: 60.0,
    aspect: 1.3333333,
};

/// Maps authored 640-wide wipe progress across the current camera plane, then emits the vertex.
pub fn bgflash_position(mut x: f32, y: f32) {
    melee_core::ct!("bgflash_position");
    let camera = ext::HSD_CObjGetCurrent();
    if !camera.is_null() {
        let width: f32 = 480.0 * ext::HSD_CObjGetAspect(camera);
        x = 320.0 + (x - 320.0) * width / 640.0;
    }
    ext::GXPosition2f32(x, y);
}

fn quad(x0: f32, y0: f32, x1: f32, y1: f32, c: [u8; 4]) {
    ext::GXBegin(GX_QUADS as u32, GX_VTXFMT0 as u32, 4);
    for (x, y) in [(x0, y0), (x1, y0), (x1, y1), (x0, y1)] {
        bgflash_position(x, y);
        ext::GXColor4u8(c[0], c[1], c[2], c[3]);
    }
}

/// 01FC08
pub unsafe extern "C" fn fn_8001FC08() {
    melee_core::ct!("fn_8001FC08");
    let data = &mut *(&raw mut lbl_80433658);
    let targets = [data.x8.r, data.x8.g, data.x8.b, data.x8.a];
    for i in 0..4 {
        let cur_val = data.x10[i];
        let val = cur_val + data.x20[i];
        let target = targets[i];
        let keep = if data.x20[i] > 0.0 { val < target as f32 } else { val > target as f32 };
        data.x10[i] = if keep { val } else { target as f32 };
    }
}

/// 01FEC4
pub unsafe extern "C" fn fn_8001FEC4(_gobj: *mut HSD_GObj, _code: i32) {
    melee_core::ct!("fn_8001FEC4");
    let data = &mut *(&raw mut lbl_80433658);
    if data.state.active() != 0 {
        return;
    }
    ext::hsd_80391A04(1.0, 1.0, 1);
    let mode = data.state.mode() as i32;
    if mode >= 5 || mode < 3 {
        let c = [data.xC.r, data.xC.g, data.xC.b, data.xC.a];
        quad(0.0, 0.0, 640.0, -480.0, c);
        return;
    }
    if data.x33 != 0 {
        if mode as u32 == 3 {
            quad(0.0, 0.0, 640.0, -480.0, [0, 0, 0, 0xFF]);
        }
        return;
    }
    let blk = [0u8, 0, 0, 0xFF];
    if data.x30 as i32 == 0 {
        if mode as u32 == 3 {
            let mut y: i32 = 0;
            while y <= data.x38 {
                let strip_h = data.x32 as i32;
                let neg_y = -y;
                let neg_yh = -(y + strip_h);
                if y == data.x38 {
                    let width = data.x34;
                    quad(0.0, neg_y as f32, width as f32, neg_yh as f32, blk);
                } else {
                    quad(0.0, neg_y as f32, 640.0, neg_yh as f32, blk);
                }
                y += data.x32 as i32;
            }
        } else {
            let p_y = data.x38;
            let mut y2 = p_y;
            while y2 <= 0x1E0 {
                let strip_h = data.x32 as i32;
                let neg_y = -y2;
                let neg_yh = -(y2 + strip_h);
                if y2 == p_y {
                    let x34 = data.x34;
                    let right = 0x280 - x34;
                    let xr = x34 + right;
                    quad(x34 as f32, neg_y as f32, xr as f32, neg_yh as f32, blk);
                } else {
                    quad(0.0, neg_y as f32, 640.0, neg_yh as f32, blk);
                }
                y2 += data.x32 as i32;
            }
        }
    }
}

/// 0204C8
pub unsafe extern "C" fn fn_800204C8() {
    melee_core::ct!("fn_800204C8");
    let data = &mut *(&raw mut lbl_80433658);
    let mode = data.state.mode() as i32;
    match mode {
        0 | 1 | 2 => {
            fn_8001FC08();
            data.xC.r = data.x10[0] as u8;
            data.xC.g = data.x10[1] as u8;
            data.xC.b = data.x10[2] as u8;
            data.xC.a = data.x10[3] as u8;
        }
        3 | 4 => {
            if data.x30 as i32 == 0 {
                let mut i = 0;
                while i < data.x3C {
                    if data.x34 < 0x280 {
                        data.x34 = data.x34.wrapping_add(data.x31 as i32);
                    } else if data.x38 < 0x1E0 {
                        data.x38 = data.x38.wrapping_add(data.x32 as i32);
                        data.x34 = 0;
                    } else {
                        data.x33 = 1;
                        return;
                    }
                    i += 1;
                }
            }
        }
        _ => {}
    }
}

/// 0205F0
pub unsafe extern "C" fn lbBgFlash_800205F0(mut duration: i32) {
    melee_core::ct!("lbBgFlash_800205F0");
    if duration < 1 {
        duration = 1;
    }
    lbBgFlash_800206D4(&raw mut lbl_804D3848, &raw mut lbl_804D3840, duration);
    lbl_80433658.state.set_mode(0);
}

/// 02063C
pub unsafe extern "C" fn lbBgFlash_8002063C(mut count: i32) {
    melee_core::ct!("lbBgFlash_8002063C");
    if count < 1 {
        count = 1;
    }
    lbBgFlash_800206D4(&raw mut lbl_804D3844, &raw mut lbl_804D3840, count);
    lbl_80433658.state.set_mode(0);
}

/// 020688
pub unsafe extern "C" fn lbBgFlash_80020688(mut count: i32) {
    melee_core::ct!("lbBgFlash_80020688");
    if count < 1 {
        count = 1;
    }
    lbBgFlash_800206D4(&raw mut lbl_804D384C, &raw mut lbl_804D3848, count);
    lbl_80433658.state.set_mode(0);
}

/// 0206D4
pub unsafe extern "C" fn lbBgFlash_800206D4(col1: *mut GXColor, col2: *mut GXColor, arg2: i32) {
    melee_core::ct!("lbBgFlash_800206D4");
    let data = &mut *(&raw mut lbl_80433658);
    let mut count = arg2;
    if count < 1 {
        count = 1;
    }
    let (c1, c2) = (*col1, *col2);
    data.state.set_active(0);
    data.state.set_mode(2);
    data.x4 = c1;
    data.xC = data.x4;
    data.x8 = c2;
    data.x10[0] = c1.r as f32;
    data.x10[1] = c1.g as f32;
    data.x10[2] = c1.b as f32;
    data.x10[3] = c1.a as f32;
    let n = count as f32;
    data.x20[0] = (c2.r as i32 - c1.r as i32) as f32 / n;
    data.x20[1] = (c2.g as i32 - c1.g as i32) as f32 / n;
    data.x20[2] = (c2.b as i32 - c1.b as i32) as f32 / n;
    data.x20[3] = (c2.a as i32 - c1.a as i32) as f32 / n;
}

/// `lbBgFlash_InitState`
pub unsafe extern "C" fn lbBgFlash_InitState(color: *mut GXColor) {
    melee_core::ct!("lbBgFlash_InitState");
    lbl_80433658.state.set_active(0);
    lbl_80433658.state.set_mode(5);
    lbl_80433658.xC = *color;
}

/// 0208B0
pub unsafe extern "C" fn fn_800208B0(arg0: u8) {
    melee_core::ct!("fn_800208B0");
    let data = &mut *(&raw mut lbl_80433658);
    data.state.set_active(0);
    data.state.set_mode(5);
    data.xC.b = 0;
    data.xC.g = 0;
    data.xC.r = 0;
    data.xC.a = arg0;
}

/// 0208EC
pub unsafe extern "C" fn lbBgFlash_800208EC(arg0: i32) {
    melee_core::ct!("lbBgFlash_800208EC");
    let flash = &mut *(&raw mut lbl_80433658);
    flash.x44 = ext::GObj_Create(0x14, 0x16, 0);
    let temp2 = ext::HSD_CObjLoadDesc((&raw mut lbl_803BB028) as *mut HSD_CObjDesc);
    let temp3 = ext::HSD_GObj_CameraKind;
    ext::HSD_GObjObject_80390A70(flash.x44, temp3, temp2 as *mut c_void);
    ext::GObj_SetupGXLinkMax(flash.x44, Some(ext::cb::HSD_GObj_803910D8), (arg0 as u16) as u8 as u32);
    (*flash.x44).gxlink_prios = 0x10000;
    flash.x40 = ext::GObj_Create(0x15, 0x16, 2);
    ext::GObj_SetupGXLink(flash.x40, Some(fn_8001FEC4), 0x10, arg0 as u8 as u32);
    ext::HSD_GObj_SetupProc(flash.x40, core::mem::transmute::<unsafe extern "C" fn(), HSD_GObjEvent>(fn_800204C8), 0);
    lbl_80433658.state.set_active(1);
    lbl_80433658.state.set_mode(0);
}

/// 0209F4
pub unsafe extern "C" fn lbBgFlash_800209F4() {
    melee_core::ct!("lbBgFlash_800209F4");
    let flash = &mut *(&raw mut lbl_80433658);
    flash.x44 = ext::GObj_Create(0x14, 0x16, 0);
    let desc = ext::HSD_CObjLoadDesc((&raw mut lbl_803BB028) as *mut HSD_CObjDesc);
    ext::HSD_GObjObject_80390A70(flash.x44, ext::HSD_GObj_CameraKind, desc as *mut c_void);
    ext::GObj_SetupGXLinkMax(flash.x44, Some(ext::cb::HSD_GObj_803910D8), 0xa);
    (*flash.x44).gxlink_prios = 0x10000;
    flash.x40 = ext::GObj_Create(0x15, 0x16, 2);
    ext::GObj_SetupGXLink(flash.x40, Some(fn_8001FEC4), 0x10, 0xa);
    ext::HSD_GObj_SetupProc(flash.x40, core::mem::transmute::<unsafe extern "C" fn(), HSD_GObjEvent>(fn_800204C8), 0);
    lbl_80433658.state.set_active(1);
    lbl_80433658.state.set_mode(0);
}
