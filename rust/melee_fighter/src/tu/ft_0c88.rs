//! src/melee/ft/ft_0C88.c: metal-box model setup and the metal (lbRefract) on/off state bits
//! (`x2226_b3..b7`, `x2030`, `smash_attrs.x2134_vibrateFrame`). No float math, no fused sites.
#![allow(non_snake_case)]

use crate::hooks as h;
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Callees outside this TU that have no port yet. Each panics until rewired.
pub mod ext {
    use crate::types::*;
    use core::ffi::c_void;
    pub fn HSD_ObjAlloc_fighter_x2040_alloc_data() -> *mut *mut HSD_DObj {
        todo!("TODO(group1 HSD) HSD_ObjAlloc(&fighter_x2040_alloc_data)")
    }
    pub fn Geno_DefineIsNone(_kind: i32) -> i32 { todo!("TODO(pc geno) Geno_DefineIsNone") }
    pub fn ftPartsPObjSetDefaultClass() { todo!("TODO(group3 ftparts) ftPartsPObjSetDefaultClass") }
    pub fn ftPartsPObjClearDefaultClass() { todo!("TODO(group3 ftparts) ftPartsPObjClearDefaultClass") }
    pub fn ft_800C85B8(_g: *mut HSD_GObj) { todo!("TODO(group3 ftmetal) ft_800C85B8") }
    pub fn ft_800C80A4(_fp: &mut Fighter) { todo!("TODO(group3 ftmetal) ft_800C80A4") }
    pub fn ft_800C8170(_fp: &mut Fighter) { todo!("TODO(group3 ftmetal) ft_800C8170") }
    pub fn ft_800C0098(_fp: &mut Fighter) { todo!("TODO(group3 ftcolanim) ft_800C0098") }
    pub fn lbRefract_80022BB8() { todo!("TODO(group1 lb) lbRefract_80022BB8") }
    pub fn lbRefract_800225D4() { todo!("TODO(group1 lb) lbRefract_800225D4") }
    pub fn lbRefSetUnuse() { todo!("TODO(group1 lb) lbRefSetUnuse") }
    #[allow(dead_code)]
    fn _unused(_: *mut c_void) {}
}

/// `ftCo_800C884C`: allocate the metal DObj list and build the metal model (TARGET_PC: a "none"
/// Geno define without a template skeleton has none).
pub fn ftco_800c884c(g: G) {
    let f = fp(g);
    f.x203C.data = ext::HSD_ObjAlloc_fighter_x2040_alloc_data();
    let kind = f.kind as i32;
    if ext::Geno_DefineIsNone(kind) != 0 && h::geno_define_template_skeleton(kind) == 0 {
        fp(g).x203C.count = 0;
        return;
    }
    ext::ftPartsPObjSetDefaultClass();
    ext::ft_800C85B8(g);
    ext::ftPartsPObjClearDefaultClass();
}

/// `ftCo_800C88A0`.
pub fn ftco_800c88a0(f: &mut Fighter) {
    f.set_x2226_b4(0);
    f.set_x2226_b5(0);
    f.smash_attrs.x2134_vibrateFrame = 0;
    f.x2030 = 0;
    f.set_x2226_b7(0);
}

fn inline_on(g: G) {
    let f = fp(g);
    f.set_x2226_b5(1);
    f.set_x2226_b7(0);
    fn_800c8b1c(g);
    ext::ft_800C80A4(fp(g));
    ext::lbRefract_80022BB8();
    fp(g).set_x221F_b2(1);
}

/// `ftCo_800C88D4(gobj, arg1, arg2)`.
pub fn ftco_800c88d4(g: G, arg1: i32, arg2: bool) {
    let f = fp(g);
    f.x2030 = arg1;
    if f.x2226_b4() == 0 || f.x2226_b7() != 0 {
        f.set_x2226_b4(1);
        inline_on(g);
        if arg2 {
            ftco_800c8b2c(fp(g), 0x7C, false);
        } else {
            ext::ft_800C0098(fp(g));
        }
    }
}

/// `ftCo_800C89A0`.
pub fn ftco_800c89a0(g: G) {
    let f = fp(g);
    ftco_800c88a0(f);
    let b3 = f.x2226_b3();
    f.set_x2226_b4(b3);
    if f.x2226_b4() != 0 {
        inline_on(g);
    }
    ext::ft_800C0098(fp(g));
}

/// `ftCo_800C8A64`.
pub fn ftco_800c8a64(g: G) {
    let f = fp(g);
    f.x2030 = 0;
    if f.x2226_b3() == 0 {
        ftco_800c88a0(f);
        ext::ft_800C0098(fp(g));
        ext::ft_800C8170(fp(g));
        ext::lbRefSetUnuse();
        fp(g).set_x221F_b2(0);
    }
}

/// `ftCo_800C8AF0`.
pub fn ftco_800c8af0(f: &mut Fighter) {
    if f.x2226_b5() != 0 {
        ext::lbRefract_800225D4();
    }
}

/// `fn_800C8B1C`.
pub fn fn_800c8b1c(g: G) {
    fp(g).smash_attrs.x2134_vibrateFrame = 0;
}

/// `ftCo_800C8B2C(fp, arg1, arg2)`.
pub fn ftco_800c8b2c(f: &mut Fighter, arg1: i32, arg2: bool) -> bool {
    if f.x2226_b6() == 0 {
        return h::ftco_800bffd0(f, arg1, arg2 as i32);
    }
    false
}

/// `ftCo_800C8B60(fp, arg1, arg2)`: `x2226_b5` is a 1-bit field, so `arg1` keeps its low bit.
pub fn ftco_800c8b60(f: &mut Fighter, arg1: u8, arg2: u8) {
    f.set_x2226_b5(arg1 & 1);
    f.smash_attrs.x2134_vibrateFrame = arg2;
}
