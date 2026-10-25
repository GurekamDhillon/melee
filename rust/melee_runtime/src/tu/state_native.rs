//! Native-layout port of `src/sysdolphin/baselib/state.c` (raw pointers over the generated `HSD_Chan` /
//! `HSD_PEDesc` records): HSD_State cached GX state (cull / blend / z / alpha / PE), the channel
//! mode per render mode, `HSD_SetupRenderModeWithCustomPE`, the invalidate table.
//! `tu::state` is the descriptor-driven renderer of the platform draw path; this is the C-shaped
//! surface, checked against the C by `tests/tu_state_native.rs` (gen/mtu).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc, dead_code)]
use core::ptr::{addr_of_mut, null_mut};
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
    fn GXSetAlphaCompare(a0: u32, a1: u8, a2: u32, a3: u32, a4: u8);
    fn GXSetAlphaUpdate(a0: u8);
    fn GXSetBlendMode(a0: u32, a1: u32, a2: u32, a3: u32);
    fn GXSetColorUpdate(a0: u8);
    fn GXSetCullMode(a0: u32);
    fn GXSetDither(a0: u8);
    fn GXSetDstAlpha(a0: u8, a1: u8);
    fn GXSetLineWidth(a0: u8, a1: u32);
    fn GXSetPointSize(a0: u8, a1: u32);
    fn GXSetZCompLoc(a0: u8);
    fn GXSetZMode(a0: u8, a1: u32, a2: u8);
    fn HSD_ClearVtxDesc() = crate::tu::pobj::HSD_ClearVtxDesc();
    fn HSD_LObjGetActiveByID(a0: u32) -> *mut HSD_LObj;
    fn HSD_LObjGetActiveByIndex(a0: i32) -> *mut HSD_LObj;
    fn HSD_LObjGetLightMaskAlpha() -> i32;
    fn HSD_LObjGetLightMaskDiffuse() -> u32;
    fn HSD_LObjGetLightMaskSpecular() -> i32;
    fn HSD_LObjGetNbActive() -> i32;
    fn HSD_LObjSetup(a0: *mut HSD_LObj, a1: _GXColor, a2: f32);
    fn HSD_MulColor(a0: *mut _GXColor, a1: *mut _GXColor, a2: *mut _GXColor);
    fn HSD_SetTevRegAll() = crate::tu::tev_native::HSD_SetTevRegAll();
    fn HSD_SetupChannel(a0: *mut HSD_Chan) = crate::tu::tev_native::HSD_SetupChannel(a0);
    fn HSD_SetupTevStage(a0: *mut _HSD_TevDesc) = crate::tu::tev_native::HSD_SetupTevStage(a0);
    fn HSD_StateAssignTev() -> i32 = crate::tu::tev_native::HSD_StateAssignTev();
    fn HSD_StateGetNumTevStages() -> i32 = crate::tu::tev_native::HSD_StateGetNumTevStages();
    fn HSD_StateSetNumChans(a0: i32) = crate::tu::tev_native::HSD_StateSetNumChans(a0);
    fn HSD_StateSetNumTevStages() = crate::tu::tev_native::HSD_StateSetNumTevStages();
    fn HSD_StateSetNumTexGens() = crate::tu::tev_native::HSD_StateSetNumTexGens();
    fn _HSD_StateInvalidateColorChannel() = crate::tu::tev_native::_HSD_StateInvalidateColorChannel();
    fn _HSD_StateInvalidateTevRegister() = crate::tu::tev_native::_HSD_StateInvalidateTevRegister();
    fn _HSD_StateInvalidateTevStage() = crate::tu::tev_native::_HSD_StateInvalidateTevStage();
    fn _HSD_StateInvalidateTexCoordGen() = crate::tu::tev_native::_HSD_StateInvalidateTexCoordGen();
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn _HSD_StateInvalidateColorChannel() { super::_HSD_StateInvalidateColorChannel() }
        pub unsafe extern "C" fn _HSD_StateInvalidateTevRegister() { super::_HSD_StateInvalidateTevRegister() }
        pub unsafe extern "C" fn _HSD_StateInvalidateTevStage() { super::_HSD_StateInvalidateTevStage() }
        pub unsafe extern "C" fn _HSD_StateInvalidateTexCoordGen() { super::_HSD_StateInvalidateTexCoordGen() }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

const GX_FALSE: i32 = 0;
const GX_TRUE: i32 = 1;

pub static mut dark_matter: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0xFF };

pub static mut state_line_width: u8 = 0;
pub static mut state_line_tex_offsets: i32 = 0;
pub static mut state_point_size: u8 = 0;
pub static mut HSD_State_804D75BC: i32 = 0;
pub static mut state_cull_mode: i32 = 0;
pub static mut state_blend_type: i32 = 0;
pub static mut state_src_factor: i32 = 0;
pub static mut state_dst_factor: i32 = 0;
pub static mut state_logic_op: i32 = 0;
pub static mut state_z_enable: u8 = 0;
pub static mut state_z_func: i32 = 0;
pub static mut state_z_update: u8 = 0;
pub static mut state_alpha_comp0: i32 = 0;
pub static mut state_alpha_ref0: u8 = 0;
pub static mut state_alpha_op: i32 = 0;
pub static mut state_alpha_comp1: i32 = 0;
pub static mut state_alpha_ref1: u8 = 0;
pub static mut state_color_update: u8 = 0;
pub static mut state_alpha_update: u8 = 0;
pub static mut state_enable_dst_alpha: u8 = 0;
pub static mut state_dst_alpha: u8 = 0;
pub static mut state_before_tex: u8 = 0;
pub static mut state_dither: u8 = 0;

/// The TU-local `matstate`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MatState {
    pub ambient: GXColor,
    pub diffuse: GXColor,
    pub specular: GXColor,
    pub alpha: u8,
    pub shininess: f32,
}
pub static mut matstate: MatState = MatState {
    ambient: GXColor { r: 0, g: 0, b: 0, a: 0 },
    diffuse: GXColor { r: 0, g: 0, b: 0, a: 0 },
    specular: GXColor { r: 0, g: 0, b: 0, a: 0 },
    alpha: 0,
    shininess: 0.0,
};

const fn chan(chan: u32, flags: u32, amb: GXColor, mat: GXColor, enable: u8, mat_src: u32, diff_fn: u32, attn_fn: u32) -> HSD_Chan {
    HSD_Chan {
        next: null_mut(),
        chan,
        flags,
        amb_color: amb,
        mat_color: mat,
        enable,
        amb_src: 0,
        mat_src,
        light_mask: 0,
        diff_fn,
        attn_fn,
        aobj: null_mut(),
    }
}
const Z4: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };
const W4: GXColor = GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };
const K4: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0xFF };
pub static mut HSD_State_80405A38: HSD_Chan = chan(1, 1, Z4, W4, 1, 0, 2, 0);
pub static mut HSD_State_80405A38_30: HSD_Chan = chan(4, 0, Z4, Z4, 0, 1, 2, 2);
pub static mut HSD_State_80405A38_60: HSD_Chan = chan(0, 0, Z4, W4, 1, 0, 2, 1);
pub static mut HSD_State_80405A38_90: HSD_Chan = chan(2, 0, K4, K4, 1, 0, 2, 1);
pub static mut HSD_State_80405A38_C0: HSD_Chan = chan(2, 0, Z4, K4, 0, 0, 0, 2);
pub static mut HSD_State_80405A38_F0: HSD_Chan = chan(4, 0, Z4, W4, 0, 0, 2, 2);

/// `invalidate_funcs`: `{ mask, func }` pairs, terminated by `HSD_STATE_NONE`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct InvalidateFunc {
    pub mask: i32,
    pub func: Option<unsafe extern "C" fn()>,
}
pub static mut invalidate_funcs: [InvalidateFunc; 8] = [
    InvalidateFunc { mask: 0x1, func: Some(_HSD_StateInvalidatePrimitive) },
    InvalidateFunc { mask: 0x2, func: Some(_HSD_StateInvalidateVtxAttr) },
    InvalidateFunc { mask: 0x4, func: Some(ext::cb::_HSD_StateInvalidateColorChannel) },
    InvalidateFunc { mask: 0x8, func: Some(ext::cb::_HSD_StateInvalidateTevStage) },
    InvalidateFunc { mask: 0x10, func: Some(ext::cb::_HSD_StateInvalidateTevRegister) },
    InvalidateFunc { mask: 0x20, func: Some(ext::cb::_HSD_StateInvalidateTexCoordGen) },
    InvalidateFunc { mask: 0x40, func: Some(_HSD_StateInvalidateRenderMode) },
    InvalidateFunc { mask: 0, func: None },
];

/// `HSD_SetupChannelMode`.
pub unsafe fn HSD_SetupChannelMode(arg0: u32) {
    let var_r3: i32 = if arg0 & 8 != 0 { 2 } else { 1 };
    ext::HSD_StateSetNumChans(var_r3);
    if arg0 & 8 != 0 {
        HSD_State_80405A38.light_mask = ext::HSD_LObjGetLightMaskSpecular() as _;
        ext::HSD_SetupChannel(addr_of_mut!(HSD_State_80405A38));
        let max = ext::HSD_LObjGetNbActive();
        let mut i = 0;
        while i < max {
            let l = ext::HSD_LObjGetActiveByIndex(i);
            if !l.is_null() {
                let var_f0 = if arg0 & 8 != 0 { matstate.shininess } else { (*l).shininess };
                ext::HSD_LObjSetup(l, (*l).color, var_f0);
            }
            i += 1;
        }
    }
    match arg0 & 7 {
        2 => ext::HSD_SetupChannel(addr_of_mut!(HSD_State_80405A38_30)),
        4 => {
            let l = ext::HSD_LObjGetActiveByID(0x100);
            if !l.is_null() && (*l).flags & 4 != 0 {
                ext::HSD_MulColor(addr_of_mut!(matstate.ambient), &mut (*l).color, addr_of_mut!(HSD_State_80405A38_60.amb_color));
            } else {
                HSD_State_80405A38_60.amb_color = dark_matter;
            }
            HSD_State_80405A38_60.light_mask = ext::HSD_LObjGetLightMaskDiffuse() as _;
            ext::HSD_SetupChannel(addr_of_mut!(HSD_State_80405A38_60));
            HSD_State_80405A38_90.light_mask = ext::HSD_LObjGetLightMaskAlpha() as _;
            if !l.is_null() && (*l).flags & 0x10 != 0 {
                HSD_State_80405A38_90.amb_color.a = (*l).color.a;
                ext::HSD_SetupChannel(addr_of_mut!(HSD_State_80405A38_90));
            } else if HSD_State_80405A38_90.light_mask != 0 {
                HSD_State_80405A38_90.amb_color.a = 0;
                ext::HSD_SetupChannel(addr_of_mut!(HSD_State_80405A38_90));
            } else {
                ext::HSD_SetupChannel(addr_of_mut!(HSD_State_80405A38_C0));
            }
        }
        _ => ext::HSD_SetupChannel(addr_of_mut!(HSD_State_80405A38_F0)),
    }
}

/// `HSD_SetupPEMode`.
pub unsafe fn HSD_SetupPEMode(flags: u32, pe: *mut HSD_PEDesc) {
    if !pe.is_null() {
        HSD_StateSetColorUpdate((*pe).flags as i32 & 1);
        HSD_StateSetAlphaUpdate((*pe).flags as i32 & 2);
        HSD_StateSetDstAlpha((*pe).flags as i32 & 4, (*pe).dst_alpha);
        HSD_StateSetBlendMode((*pe).r#type as i32, (*pe).src_factor as i32, (*pe).dst_factor as i32, (*pe).logic_op as i32);
        HSD_StateSetZMode((*pe).flags as i32 & 0x10, (*pe).z_comp as i32, (*pe).flags as i32 & 0x20);
        HSD_StateSetZCompLoc((*pe).flags as i32 & 8);
        HSD_StateSetAlphaCompare((*pe).alpha_comp0 as i32, (*pe).ref0, (*pe).alpha_op as i32, (*pe).alpha_comp1 as i32, (*pe).ref1);
        HSD_StateSetDither((*pe).flags as i32 & 0x40);
        return;
    }
    HSD_StateSetColorUpdate(1);
    HSD_StateSetAlphaUpdate(0);
    HSD_StateSetDstAlpha(0, 0);
    let var_r3 = if flags & 0x4000_0000 != 0 { 1 } else { 0 };
    HSD_StateSetBlendMode(var_r3, 4, 5, 0xF);
    let var_r3_2 = if flags & 0x2000_0000 != 0 { GX_FALSE } else { GX_TRUE };
    HSD_StateSetZMode(1, if flags & 0x0800_0000 != 0 { 7 } else { 3 }, var_r3_2);
    if flags & 0x2000_0000 == 0 && flags & 0x4000_0000 != 0 {
        HSD_StateSetZCompLoc(0);
        HSD_StateSetAlphaCompare(4, 0, 0, 4, 0);
    } else {
        HSD_StateSetZCompLoc(1);
        HSD_StateSetAlphaCompare(7, 0, 0, 7, 0);
    }
    HSD_StateSetDither(0);
}

unsafe fn setupTevMode_last() {
    if ext::HSD_StateGetNumTevStages() == 0 {
        let mut tevdesc: HSD_TevDesc = core::mem::zeroed();
        tevdesc.flags = 0;
        tevdesc.stage = ext::HSD_StateAssignTev() as u32;
        tevdesc.coord = 0xFF;
        tevdesc.map = 0xFF; // GX_TEXMAP_NULL
        tevdesc.color = 4; // GX_COLOR0A0
        tevdesc.u.tevop.tevmode = 4; // GX_PASSCLR
        ext::HSD_SetupTevStage(&mut tevdesc);
    }
}

/// `HSD_SetupRenderModeWithCustomPE`.
pub unsafe fn HSD_SetupRenderModeWithCustomPE(rendermode: u32, pe: *mut HSD_PEDesc) {
    setupTevMode_last();
    HSD_SetupPEMode(rendermode, pe);
    ext::HSD_SetTevRegAll();
    ext::HSD_StateSetNumTevStages();
    ext::HSD_StateSetNumTexGens();
    HSD_SetupChannelMode(rendermode);
}

/// `HSD_SetupRenderMode`.
pub unsafe fn HSD_SetupRenderMode(rendermode: u32) {
    HSD_SetupRenderModeWithCustomPE(rendermode, null_mut());
}

/// `HSD_SetMaterialColor`.
pub unsafe fn HSD_SetMaterialColor(ambient: GXColor, diffuse: GXColor, specular: GXColor, alpha: f32) {
    matstate.ambient = ambient;
    matstate.diffuse = diffuse;
    matstate.specular = specular;
    matstate.alpha = (crate::cabi::f2u32(255.0f64 * alpha as f64) as u8);
}

/// `HSD_SetMaterialShininess`.
pub unsafe fn HSD_SetMaterialShininess(shininess: f32) {
    matstate.shininess = shininess;
}

/// `HSD_StateSetLineWidth`.
pub unsafe fn HSD_StateSetLineWidth(arg0: u8, arg1: i32) {
    if state_line_width != arg0 || state_line_tex_offsets != arg1 {
        ext::GXSetLineWidth(arg0, arg1 as u32);
        state_line_width = arg0;
        state_line_tex_offsets = arg1;
    }
}

/// `HSD_StateSetCullMode`.
pub unsafe fn HSD_StateSetCullMode(mode: i32) {
    if state_cull_mode != mode {
        ext::GXSetCullMode(mode as u32);
        state_cull_mode = mode;
    }
}

/// `HSD_StateSetBlendMode`.
pub unsafe fn HSD_StateSetBlendMode(ty: i32, src_factor: i32, dst_factor: i32, op: i32) {
    if state_blend_type != ty || state_src_factor != src_factor || state_dst_factor != dst_factor || state_logic_op != op {
        ext::GXSetBlendMode(ty as u32, src_factor as u32, dst_factor as u32, op as u32);
        state_blend_type = ty;
        state_src_factor = src_factor;
        state_dst_factor = dst_factor;
        state_logic_op = op;
    }
}

/// `HSD_StateSetZMode`.
pub unsafe fn HSD_StateSetZMode(arg0: i32, arg1: i32, arg2: i32) {
    let arg0 = if arg0 != 0 { GX_TRUE } else { GX_FALSE };
    let arg2 = if arg2 != 0 { GX_TRUE } else { GX_FALSE };
    if state_z_enable as i32 != arg0 || state_z_func != arg1 || state_z_update as i32 != arg2 {
        ext::GXSetZMode(arg0 as u8, arg1 as u32, arg2 as u8);
        state_z_enable = arg0 as u8;
        state_z_func = arg1;
        state_z_update = arg2 as u8;
    }
}

/// `HSD_StateSetPointSize`.
pub unsafe fn HSD_StateSetPointSize(arg0: u8, arg1: i32) {
    if state_point_size != arg0 || HSD_State_804D75BC != arg1 {
        ext::GXSetPointSize(arg0, arg1 as u32);
        state_point_size = arg0;
        HSD_State_804D75BC = arg1;
    }
}

/// `HSD_StateSetAlphaCompare`.
pub unsafe fn HSD_StateSetAlphaCompare(arg0: i32, arg1: u8, arg2: i32, arg3: i32, arg4: u8) {
    if state_alpha_comp0 != arg0 || state_alpha_ref0 != arg1 || state_alpha_op != arg2 || state_alpha_comp1 != arg3 || state_alpha_ref1 != arg4 {
        ext::GXSetAlphaCompare(arg0 as u32, arg1, arg2 as u32, arg3 as u32, arg4);
        state_alpha_comp0 = arg0;
        state_alpha_ref0 = arg1;
        state_alpha_op = arg2;
        state_alpha_comp1 = arg3;
        state_alpha_ref1 = arg4;
    }
}

/// `HSD_StateSetColorUpdate`.
pub unsafe fn HSD_StateSetColorUpdate(arg0: i32) {
    let arg0 = if arg0 != 0 { GX_TRUE } else { GX_FALSE };
    if state_color_update as i32 != arg0 {
        ext::GXSetColorUpdate(arg0 as u8);
        state_color_update = arg0 as u8;
    }
}

/// `HSD_StateSetAlphaUpdate`.
pub unsafe fn HSD_StateSetAlphaUpdate(arg0: i32) {
    let arg0 = if arg0 != 0 { GX_TRUE } else { GX_FALSE };
    if state_alpha_update as i32 != arg0 {
        ext::GXSetAlphaUpdate(arg0 as u8);
        state_alpha_update = arg0 as u8;
    }
}

/// `HSD_StateSetDstAlpha`.
pub unsafe fn HSD_StateSetDstAlpha(arg0: i32, arg1: u8) {
    let arg0 = if arg0 != 0 { GX_TRUE } else { GX_FALSE };
    if state_enable_dst_alpha as i32 != arg0 || state_dst_alpha != arg1 {
        ext::GXSetDstAlpha(arg0 as u8, arg1);
        state_enable_dst_alpha = arg0 as u8;
        state_dst_alpha = arg1;
    }
}

/// `HSD_StateSetZCompLoc`.
pub unsafe fn HSD_StateSetZCompLoc(arg0: i32) {
    let arg0 = if arg0 != 0 { GX_TRUE } else { GX_FALSE };
    if state_before_tex as i32 != arg0 {
        ext::GXSetZCompLoc(arg0 as u8);
        state_before_tex = arg0 as u8;
    }
}

/// `HSD_StateSetDither`.
pub unsafe fn HSD_StateSetDither(arg0: i32) {
    let arg0 = if arg0 != 0 { GX_TRUE } else { GX_FALSE };
    if state_dither as i32 != arg0 {
        ext::GXSetDither(arg0 as u8);
        state_dither = arg0 as u8;
    }
}

/// `_HSD_StateInvalidatePrimitive`.
pub unsafe extern "C" fn _HSD_StateInvalidatePrimitive() {
    state_line_width = 0;
    state_point_size = 0;
    state_cull_mode = -1;
}

/// `_HSD_StateInvalidateVtxAttr`.
pub unsafe extern "C" fn _HSD_StateInvalidateVtxAttr() {
    ext::HSD_ClearVtxDesc();
}

/// `_HSD_StateInvalidateRenderMode`.
pub unsafe extern "C" fn _HSD_StateInvalidateRenderMode() {
    state_blend_type = -1;
    state_src_factor = -1;
    state_dst_factor = -1;
    state_logic_op = -1;
    state_z_enable = 0xFF;
    state_z_func = -1;
    state_z_update = 0xFF;
    state_alpha_comp0 = -1;
    state_alpha_ref0 = 0;
    state_alpha_op = -1;
    state_alpha_comp1 = -1;
    state_alpha_ref1 = 0;
    state_color_update = 0xFF;
    state_alpha_update = 0xFF;
    state_enable_dst_alpha = 0xFF;
    state_dst_alpha = 0;
    state_before_tex = 0xFF;
    state_dither = 0xFF;
}

/// `HSD_StateInvalidate`.
pub unsafe fn HSD_StateInvalidate(mask: i32) {
    let mut i = 0;
    while invalidate_funcs[i].mask != 0 {
        if mask & invalidate_funcs[i].mask != 0 {
            (invalidate_funcs[i].func.unwrap())();
        }
        i += 1;
    }
}
