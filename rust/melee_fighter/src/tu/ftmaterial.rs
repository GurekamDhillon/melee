//! src/melee/ft/ftmaterial.c: the fighter MObj class (`ftMObj`, every fighter MObj is
//! `hsdChangeClass`ed onto it) whose `setup` applies the fighter's colour overlay, metal / KO
//! material swap, refraction and light overlay to the TEV, plus `ftMaterial_800BFB4C` (set the
//! diffuse colour of every material of a fighter). The TARGET_PC build uses the file-static TEV
//! templates, not the class-info copy.
//!
//! GX state calls are the `hooks!` of `melee_runtime::tu::mobj::native::ext` /
//! `melee_runtime::tu::texp::ext` (unset they panic with the C name). No fused sites
//! (fma_sites.py).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use crate::types::*;
use core::ffi::{c_char, c_void};
use core::mem::size_of;
use core::ptr::null_mut;
use melee_runtime::tu::mobj::native::ext as mx;
use melee_runtime::tu::texp::ext as tx;

/// Calls out of this TU: settable hooks (the oracle's recording stubs install into `ext::hook`); unset they run the
/// ported callee.
#[allow(non_snake_case, unused_variables)]
pub mod ext {
    use crate::types::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $d:expr;)*) => {
            pub mod hook {
                use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(unreachable_code, unused_variables)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => $d,
                }
            } )*
        };
    }
    ext_fns! {
        fn HSD_JObjGetDObj(_a0: *mut core::ffi::c_void) -> *mut core::ffi::c_void = { unsafe { (melee_runtime::tu::jobj_ptr::HSD_JObjGetDObj(_a0 as _)) as _ } };
        fn HSD_SetMaterialColor(_a0: GXColor, _a1: GXColor, _a2: GXColor, _a3: f32) = { super::mx::HSD_SetMaterialColor(_a0, _a1, _a2, _a3) };
        fn HSD_SetMaterialShininess(_a0: f32) = { super::mx::HSD_SetMaterialShininess(_a0) };
        fn HSD_StateAssignTev() -> i32 = { unsafe { super::tx::HSD_StateAssignTev() } };
        fn HSD_StateInitTev() = { super::mx::HSD_StateInitTev() };
        fn HSD_TObjSetup(_a0: *mut core::ffi::c_void) = { unsafe { let _ = super::mx::HSD_TObjSetup(_a0 as _); } };
        fn HSD_TObjSetupTextureCoordGen(_a0: *mut core::ffi::c_void) = { unsafe { let _ = super::mx::HSD_TObjSetupTextureCoordGen(_a0 as _); } };
        fn ftCo_8009F75C(_a0: *mut core::ffi::c_void, _a1: i32) = { unsafe { let _ = crate::tu::ftCo_09f4::ftCo_8009F75C(_a0 as _, _a1 != 0); } };
        fn ftCo_800C0658(_a0: *mut core::ffi::c_void) -> *mut core::ffi::c_void = { unsafe { (crate::ft0df::ftco_800c0658(&mut *(_a0 as *mut Fighter))) as _ } };
        fn hsdInitClassInfo(_a0: *mut core::ffi::c_void, _a1: *mut core::ffi::c_void, _a2: *mut core::ffi::c_void, _a3: *mut core::ffi::c_void, _a4: i32, _a5: i32) = { unsafe { let _ = melee_runtime::tu::class::hsdInitClassInfo(_a0 as _, _a1 as _, _a2 as _, _a3 as _, _a4 as _, _a5 as _); } };
        fn lbGetFreeColorRegister(_a0: i32, _a1: *mut core::ffi::c_void, _a2: *mut core::ffi::c_void) -> i32 = { unsafe { (melee_runtime::tu::lb_00B0::lbGetFreeColorRegister(_a0 as _, _a1 as _, _a2 as _)) as _ } };
        fn lbRefract_80022998(_a0: *mut core::ffi::c_void, _a1: u32, _a2: i32) = { unsafe { let _ = melee_runtime::tu::lbrefract::lbRefract_80022998(_a0 as _, _a1 as _, _a2 as _); } };
        fn lb_8000CC8C(_a0: i32) -> i32 = { unsafe { (melee_runtime::tu::lb_00B0::lb_8000CC8C(_a0 as _)) as _ } };
        fn lb_8000CCA4(_a0: i32) -> i32 = { unsafe { (melee_runtime::tu::lb_00B0::lb_8000CCA4(_a0 as _)) as _ } };
        fn lb_8000CD90(_a0: i32) -> i32 = { unsafe { (melee_runtime::tu::lb_00B0::lb_8000CD90(_a0 as _)) as _ } };
        fn lb_8000CDA8(_a0: i32) -> i32 = { unsafe { (melee_runtime::tu::lb_00B0::lb_8000CDA8(_a0 as _)) as _ } };
        fn HSD_TExpSetReg(_a0: *mut c_void) = { unsafe { melee_runtime::tu::texp::HSD_TExpSetReg(_a0 as _) } };
        fn HSD_SetupTevStage(_a0: *mut c_void) = { unsafe { super::tx::HSD_SetupTevStage(&mut *(_a0 as *mut HSD_TevDesc)) } };
        fn HSD_SetupRenderModeWithCustomPE(_a0: u32, _a1: *mut c_void) = { super::mx::HSD_SetupRenderModeWithCustomPE(_a0, _a1 as _) };
    }
}

pub static mut ftMObj: HSD_MObjInfo = {
    let mut v: HSD_MObjInfo = unsafe { core::mem::zeroed() };
    v.parent.head.info_init = Some(ftMaterial_800BF260);
    v
};

/// `&ftMObj`.
pub fn ftMObj_ptr() -> *mut HSD_MObjInfo {
    melee_core::ct!("ftMObj_ptr");
    &raw mut ftMObj
}

/// `ftMaterial_803C69D0`: the TEV stage that adds a constant colour to PREV.
fn tevdesc_template() -> HSD_TevDesc {
    let mut d: HSD_TevDesc = unsafe { core::mem::zeroed() };
    d.next = null_mut();
    d.flags = TEVCONF_MODE as u32;
    d.stage = GX_TEVSTAGE0 as u32;
    d.coord = GX_TEXCOORD_NULL as u32;
    d.map = GX_TEXCOORD_NULL as u32;
    d.color = GX_COLOR_NULL as u32;
    let c = unsafe { &mut d.u.tevconf };
    c.clr_op = GX_TEV_ADD;
    c.clr_a = GX_CC_CPREV;
    c.clr_b = GX_CC_ZERO;
    c.clr_c = GX_CC_ZERO;
    c.clr_d = GX_CC_ZERO;
    c.clr_scale = GX_CS_SCALE_1;
    c.clr_bias = GX_TB_ZERO;
    c.clr_clamp = 1; // GX_ENABLE
    c.clr_out_reg = GX_TEVPREV;
    c.alpha_op = GX_TEV_ADD;
    c.alpha_a = GX_CA_ZERO;
    c.alpha_b = GX_CA_ZERO;
    c.alpha_c = GX_CA_ZERO;
    c.alpha_d = GX_CA_APREV;
    c.alpha_scale = GX_CS_SCALE_1;
    c.alpha_bias = GX_TB_ZERO;
    c.alpha_clamp = 0; // GX_DISABLE
    c.alpha_out_reg = GX_TEVPREV;
    c.mode = GX_TC_LINEAR;
    c.ras_swap = GX_TEV_SWAP0;
    c.tex_swap = GX_TEV_SWAP0;
    c.kcsel = GX_TEV_KCSEL_1;
    c.kasel = GX_TEV_KASEL_1;
    d
}

/// `ftMaterial_803C6A44`: a constant-colour expression leaf.
fn cnst_template() -> HSD_TECnst {
    let mut c: HSD_TECnst = unsafe { core::mem::zeroed() };
    c.r#type = HSD_TE_CNST;
    c.next = null_mut();
    c.val = null_mut();
    c.comp = HSD_TE_RGB;
    c.ctype = HSD_TE_U8;
    c.reg = 0xFF;
    c.idx = 0xFF;
    c.r#ref = 0;
    c.range = 0;
    c
}

pub unsafe extern "C" fn ftMaterial_800BF260() {
    melee_core::ct!("ftMaterial_800BF260");
    ext::hsdInitClassInfo(crate::mtu_ptr::ptr(&((&raw mut ftMObj).cast::<c_void>())), crate::mtu_ptr::ptr(&((&raw mut melee_runtime::tu::mobj::native::hsdMObj).cast::<c_void>())), crate::mtu_ptr::ptr(&(c"sysdolphin_base_library".as_ptr() as *mut c_char)), crate::mtu_ptr::ptr(&(c"ft_mobj".as_ptr() as *mut c_char)), (size_of::<HSD_MObjInfo>() as i32) as i32, (size_of::<HSD_MObj>() as i32) as i32);
    // The C stores a three-argument function in the two-argument setup slot.
    ftMObj.setup = Some(core::mem::transmute::<
        unsafe extern "C-unwind" fn(*mut HSD_MObj, u32, u32),
        unsafe extern "C" fn(*mut HSD_MObj, u32),
    >(ftMaterial_800BF2B8));
}

/// `ftMaterial_800BF2B8(mobj, rendermode, unused)`: the fighter MObj's TEV setup.
pub unsafe extern "C-unwind" fn ftMaterial_800BF2B8(mut mobj: *mut HSD_MObj, mut rendermode: u32, _unused: u32) {
    let fp = (*melee_runtime::tu::pobj::ext::HSD_GObj_804D7814).user_data as *mut Fighter;

    if (*fp).x2226_b5() != 0 {
        ext::lbRefract_80022998(crate::mtu_ptr::ptr(&(mobj)), (rendermode) as u32, ((*fp).smash_attrs.x2134_vibrateFrame as i32) as i32);
        return;
    }

    if (*fp).x2223_b2() == 0 && ((*fp).x2228_b0() == 0 || (*fp).x2224_b0() == 0) {
        if (*fp).is_metal() != 0 {
            mobj = crate::tu::ftco_800c7ca0::ft_804D6580;
        } else if (*fp).x2227_b3() != 0 {
            mobj = crate::tu::ft_0c8c::ft_804D6588;
        }
    }

    ext::HSD_StateInitTev();

    rendermode = (*mobj).rendermode;
    let mat = (*mobj).mat;
    ext::HSD_SetMaterialColor((*mat).ambient, (*mat).diffuse, (*mat).specular, (*mat).alpha);
    if rendermode & RENDER_SPECULAR as u32 != 0 {
        ext::HSD_SetMaterialShininess(((*mat).shininess) as f32);
    }
    let mut cur_tobj: *mut *mut HSD_TObj = null_mut();
    let mut tobj = (*mobj).tobj;
    if rendermode & RENDER_SHADOW as u32 != 0 && !melee_runtime::tu::mobj::native::tobj_shadows.is_null() {
        cur_tobj = &mut tobj;
        while !(*cur_tobj).is_null() {
            cur_tobj = &mut (**cur_tobj).next;
        }
        *cur_tobj = melee_runtime::tu::mobj::native::tobj_shadows;
    }
    let toon = melee_runtime::tu::mobj::native::tobj_toon;
    if rendermode & RENDER_TOON as u32 != 0 && !toon.is_null() && !(*toon).imagedesc.is_null() {
        (*toon).next = tobj;
        tobj = toon;
    }
    ext::HSD_TObjSetup(crate::mtu_ptr::ptr(&(tobj)));
    ext::HSD_TObjSetupTextureCoordGen(crate::mtu_ptr::ptr(&(tobj)));
    // HSD_MOBJ_METHOD(mobj)->setup_tev(mobj, tobj, rendermode)
    let info = (*mobj).parent.class_info as *mut HSD_MObjInfo;
    ((*info).setup_tev.unwrap())(mobj, tobj, rendermode);

    if (*fp).x61D != 0xFF {
        rendermode |= RENDER_NO_ZUPDATE as u32 | RENDER_XLU as u32;
    }
    let mut texp: HSD_TExp = core::mem::zeroed();
    let texp1 = ftMaterial_800BF534(fp, mobj, &mut texp, rendermode);
    ftMaterial_800BF6BC(fp, mobj, texp1);
    if (*fp).x2223_b2() != 0 && (*fp).x2223_b3() == 0 {
        rendermode |= RENDER_NO_ZUPDATE as u32;
    }
    let mut pe: HSD_PEDesc = core::mem::zeroed();
    let pe_p: *mut HSD_PEDesc;
    if (*fp).x2223_b3() != 0 && (*fp).x61D == 0xFF {
        pe.flags = (1 << 3) | (1 << 4) | (1 << 5);
        pe.dst_alpha = 0;
        pe.r#type = 0;
        pe.src_factor = 4;
        pe_p = &mut pe;
        pe.dst_factor = 5;
        pe.logic_op = 15;
        pe.z_comp = 3;
        pe.alpha_comp0 = 7;
        pe.ref0 = 0;
        pe.alpha_op = 0;
        pe.alpha_comp1 = 7;
        pe.ref1 = 0;
    } else {
        pe_p = (*mobj).pe;
    }
    ext::HSD_SetupRenderModeWithCustomPE(rendermode, pe_p.cast());
    if texp1.is_null() {
        ext::ftCo_8009F75C(crate::mtu_ptr::ptr(&(fp)), (true) as i32);
    }
    if !cur_tobj.is_null() {
        *cur_tobj = null_mut();
    }
}

/// `ftMaterial_800BF534(fp, mobj, texp, rendermode)`: the light-overlay constant and TEV stage.
pub unsafe fn ftMaterial_800BF534(fp: *mut Fighter, mobj: *mut HSD_MObj, texp: *mut HSD_TExp, rendermode: u32) -> *mut HSD_TExp {
    melee_core::ct!("ftMaterial_800BF534");
    let overlay = (ext::ftCo_800C0658(crate::mtu_ptr::ptr(&(&mut *fp))) as *mut ColorOverlay);
    if (*overlay).x7C_flag2() != 0 && (*overlay).x7C_light_enable() != 0 {
        if rendermode & RENDER_XLU as u32 == 0 && (*fp).x2223_b2() == 0 {
            (*texp).cnst = cnst_template();
            let reg = ext::lbGetFreeColorRegister((0) as i32, crate::mtu_ptr::ptr(&(mobj)), crate::mtu_ptr::ptr(&(null_mut::<u8>())));
            if reg == -1 {
                panic!("ftmaterial.c:240 can't find free color register!");
            }
            (*texp).cnst.reg = reg as u8;
            (*texp).cnst.val = (&raw mut (*overlay).x50_light_color).cast();
            ext::HSD_TExpSetReg(texp.cast());

            let mut sp_tevdesc = tevdesc_template();
            sp_tevdesc.stage = ext::HSD_StateAssignTev() as u32;
            sp_tevdesc.color = 2;
            sp_tevdesc.u.tevconf.clr_a = GX_CC_ZERO;
            sp_tevdesc.u.tevconf.clr_b = ext::lb_8000CC8C((reg) as i32) as _;
            sp_tevdesc.u.tevconf.clr_c = GX_CC_RASA;
            sp_tevdesc.u.tevconf.clr_d = GX_CC_CPREV;
            if reg < 4 {
                sp_tevdesc.u.tevconf.kcsel = ext::lb_8000CCA4((reg) as i32) as _;
            }
            ext::HSD_SetupTevStage((&raw mut sp_tevdesc).cast());
            return texp;
        }
        ext::ftCo_8009F75C(crate::mtu_ptr::ptr(&(fp)), (false) as i32);
    }
    null_mut()
}

/// `ftMaterial_800BF6BC(fp, mobj, texp)`: the fighter colour overlay as a TEV blend stage.
pub unsafe fn ftMaterial_800BF6BC(fp: *mut Fighter, mobj: *mut HSD_MObj, texp: *mut HSD_TExp) {
    melee_core::ct!("ftMaterial_800BF6BC");
    if (*fp).x2223_b3() != 0 {
        return;
    }
    let overlay = (ext::ftCo_800C0658(crate::mtu_ptr::ptr(&(&mut *fp))) as *mut ColorOverlay);
    let mut chk1 = false;
    let mut sp168: GXColor = core::mem::zeroed();
    let mut var_r5 = (*fp).x61A_controller_index as usize;
    if (*fp).x2228_b0() != 0 && (*fp).x2224_b0() != 0 {
        if (*fp).is_metal() != 0 {
            var_r5 = 4;
        } else if (*fp).x2227_b3() != 0 {
            var_r5 = 5;
        }
    }
    if (*fp).x2223_b2() != 0 {
        chk1 = true;
        sp168 = (*fp).x610_color_rgba[1];
    } else if var_r5 != 0 {
        if (*overlay).x7C_color_enable() != 0 {
            let fp_color = (*fp).x610_color_rgba[0];
            let color_hex = (*overlay).x2C_hex;
            let temp_alpha: u32 = (((0xFF - fp_color.a as i32) * (0xFF - color_hex.a as i32)) / 255) as u32;
            if temp_alpha as i32 == 0xFF {
                sp168 = color_hex;
            } else {
                let inv_alpha: i32 = 0xFF - temp_alpha as i32;
                let mut temp_r8: i32 = fp_color.r as i32;
                temp_r8 += (color_hex.a as i32 * (color_hex.r as i32 - temp_r8)) / 255;
                let temp_r7 = temp_r8 * 0xFF;
                sp168.r = (temp_r7 / inv_alpha) as u8;
                if sp168.r != 0 {
                    sp168.a = (temp_r7 / sp168.r as i32) as u8;
                } else {
                    sp168.a = (((inv_alpha - temp_r8) * 0xFF) / 255) as u8;
                }
                let temp_r8_3 = fp_color.g as i32;
                sp168.g = (((temp_r8_3 + ((color_hex.a as i32 * (color_hex.g as i32 - temp_r8_3)) / 255)) * 0xFF) / inv_alpha) as u8;
                let temp_r6 = fp_color.b as i32;
                sp168.b = (((temp_r6 + ((color_hex.a as i32 * (color_hex.b as i32 - temp_r6)) / 255)) * 0xFF) / inv_alpha) as u8;
            }
            chk1 = true;
        } else {
            chk1 = true;
            // `((GXColor*) &p_ftCommonData->x6D8)[var_r5]`
            let base = (&raw const crate::hooks::common_data().x6D8) as *const GXColor;
            sp168 = *base.add(var_r5);
        }
    } else if (*overlay).x7C_color_enable() != 0 {
        chk1 = true;
        sp168 = (*overlay).x2C_hex;
    }
    if !chk1 {
        return;
    }
    let mut sp_cnst1 = cnst_template();
    let reg1 = ext::lbGetFreeColorRegister((0) as i32, crate::mtu_ptr::ptr(&(mobj)), crate::mtu_ptr::ptr(&(texp)));
    if reg1 == -1 {
        panic!("ftmaterial.c:352 can't find free color register!");
    }
    sp_cnst1.reg = reg1 as u8;
    sp_cnst1.val = (&raw mut sp168).cast();
    let p1 = (&raw mut sp_cnst1) as *mut HSD_TExp;
    ext::HSD_TExpSetReg(p1.cast());
    sp_cnst1.next = texp;
    let var_r3 = if reg1 < 4 { 4 } else { 0 };
    let reg2 = ext::lbGetFreeColorRegister((var_r3) as i32, crate::mtu_ptr::ptr(&(mobj)), crate::mtu_ptr::ptr(&(p1)));
    if reg2 == -1 {
        panic!("ftmaterial.c:366 can't find free color ratio register!");
    }
    let mut sp_cnst2 = cnst_template();
    if (*fp).x61D != 0xFF {
        sp_cnst2.reg = reg2 as u8;
        sp_cnst2.comp = 5;
        sp_cnst2.idx = 3;
        sp_cnst2.val = (&raw mut (*fp).x61D).cast();
        sp_cnst1.next = (&raw mut sp_cnst2) as *mut HSD_TExp;
    } else {
        sp_cnst1.next = null_mut();
    }
    sp_cnst1.reg = reg2 as u8;
    let mut color = GXColor { r: sp168.a, g: sp168.a, b: sp168.a, a: 0 };
    sp_cnst1.val = (&raw mut color).cast();
    ext::HSD_TExpSetReg(p1.cast());

    let mut sp_tevdesc = tevdesc_template();
    sp_tevdesc.stage = ext::HSD_StateAssignTev() as u32;
    sp_tevdesc.u.tevconf.clr_b = ext::lb_8000CC8C((reg1) as i32) as _;
    sp_tevdesc.u.tevconf.clr_c = ext::lb_8000CC8C((reg2) as i32) as _;
    if reg1 < 4 {
        sp_tevdesc.u.tevconf.kcsel = ext::lb_8000CCA4((reg1) as i32) as _;
    } else if reg2 < 4 {
        sp_tevdesc.u.tevconf.kcsel = ext::lb_8000CCA4((reg2) as i32) as _;
    }
    if (*fp).x61D != 0xFF {
        sp_tevdesc.u.tevconf.alpha_d = ext::lb_8000CD90((reg2) as i32) as _;
        if reg2 < 4 {
            sp_tevdesc.u.tevconf.kasel = ext::lb_8000CDA8((reg2) as i32) as _;
        }
    }
    ext::HSD_SetupTevStage((&raw mut sp_tevdesc).cast());
}

/// `ftMaterial_800BFB4C(gobj, diffuse)`: set the diffuse colour of every material of the fighter's model.
pub unsafe fn ftMaterial_800BFB4C(gobj: *mut HSD_GObj, diffuse: *const GXColor) {
    melee_core::ct!("ftMaterial_800BFB4C");
    let mut cur = (*gobj).hsd_obj as *mut HSD_JObj;
    while !cur.is_null() {
        let mut dobj = (ext::HSD_JObjGetDObj(crate::mtu_ptr::ptr(&(cur))) as *mut HSD_DObj);
        while !dobj.is_null() {
            let mobj = (*dobj).mobj;
            if !mobj.is_null() && !(*mobj).mat.is_null() {
                (*(*mobj).mat).diffuse = *diffuse;
            }
            dobj = (*dobj).next;
        }
        if (*cur).flags & JOBJ_INSTANCE as u32 == 0 && !(*cur).child.is_null() {
            cur = (*cur).child;
            continue;
        }
        if !(*cur).next.is_null() {
            cur = (*cur).next;
        } else {
            loop {
                let parent = (*cur).parent;
                if parent.is_null() {
                    cur = null_mut();
                } else if !(*parent).next.is_null() {
                    cur = (*parent).next;
                } else {
                    cur = parent;
                    continue;
                }
                break;
            }
        }
    }
}

#[allow(dead_code)]
fn _unused(_: *mut c_void) {}
