//! Port of src/melee/it/itmaterial.c: the item material (`it_mobj`) class, its setup callback (TEV color
//! overlay and shadow/toon texture chaining) and the per-material diffuse color walk over a jobj tree.
//! Callees of other TUs go through `ext` (unset hooks panic "TODO").
//! Checked against the C by the state oracle `tests/tu_itmaterial.rs` (gen/mtu).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_mut, unused_assignments, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(unreachable_code, non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjGetDObj(a0: *mut HSD_JObj) -> *mut HSD_DObj => unsafe { crate::tu::jobj::HSD_JObjGetDObj(a0) as _ };
    fn HSD_SetMaterialColor(a0: _GXColor, a1: _GXColor, a2: _GXColor, a3: f32);
    fn HSD_SetMaterialShininess(a0: f32);
    fn HSD_SetupRenderModeWithCustomPE(a0: u32, a1: *mut HSD_PEDesc);
    fn HSD_SetupTevStage(a0: *mut _HSD_TevDesc);
    fn HSD_StateAssignTev() -> i32;
    fn HSD_StateInitTev();
    fn HSD_TExpSetReg(a0: *mut HSD_TExp) => unsafe { melee_runtime::tu::texp::HSD_TExpSetReg(a0) };
    fn HSD_TObjSetup(a0: *mut HSD_TObj) => unsafe { melee_runtime::tu::tobj::HSD_TObjSetup(a0) };
    fn HSD_TObjSetupTextureCoordGen(a0: *mut HSD_TObj) => unsafe { melee_runtime::tu::tobj::HSD_TObjSetupTextureCoordGen(a0) };
    fn ftCo_800C0674(a0: *mut HSD_GObj) -> *mut ColorOverlay;
    fn hsdInitClassInfo(a0: *mut _HSD_ClassInfo, a1: *mut _HSD_ClassInfo, a2: *mut i8, a3: *mut i8, a4: i32, a5: i32) => unsafe { melee_runtime::tu::class::hsdInitClassInfo(a0 as _, a1 as _, a2 as _, a3 as _, a4, a5) };
    fn lbGetFreeColorRegister(a0: i32, a1: *mut HSD_MObj, a2: *mut HSD_TExp) -> i32 => unsafe { melee_runtime::tu::lb_00B0::lbGetFreeColorRegister(a0, a1, a2) };
    fn lb_8000CC8C(a0: i32) -> i32 => unsafe { melee_runtime::tu::lb_00B0::lb_8000CC8C(a0) };
    fn lb_8000CCA4(a0: i32) -> i32 => unsafe { melee_runtime::tu::lb_00B0::lb_8000CCA4(a0) };
    fn lb_8000CD90(a0: i32) -> i32 => unsafe { melee_runtime::tu::lb_00B0::lb_8000CD90(a0) };
    fn lb_8000CDA8(a0: i32) -> i32 => unsafe { melee_runtime::tu::lb_00B0::lb_8000CDA8(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObj_804D7814: *mut HSD_GObj = unsafe { core::mem::zeroed() };
    pub static mut hsdMObj: HSD_MObjInfo = unsafe { core::mem::zeroed() };
    pub static mut tobj_shadows: *mut HSD_TObj = unsafe { core::mem::zeroed() };
    pub static mut tobj_toon: *mut HSD_TObj = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

const RENDER_SHADOW: u32 = melee_types::RENDER_SHADOW as u32; // 1 << 26
const RENDER_TOON: u32 = melee_types::RENDER_TOON as u32; // 1 << 12
const RENDER_NO_ZUPDATE: u32 = 1 << 29;
const RENDER_BLENDING: u32 = (1 << 30) | RENDER_NO_ZUPDATE; // RENDER_XLU | RENDER_NO_ZUPDATE

/// `struct it_MObjInfo` (itmaterial.c). Not a melee_types record: the first five members mirror
/// `HSD_MObjInfo`, then the item-specific template data. GC offsets: setup +3C, load +40, make_texp +44,
/// setup_tev +48, unset +4C, tevdesc_tmpl +50, texp_tmpl +C4.
#[repr(C)]
pub struct it_MObjInfo {
    pub parent: HSD_ClassInfo,
    pub setup: it_MObjSetupFunc,
    pub load: Option<unsafe extern "C" fn(*mut HSD_MObj, *mut _HSD_MObjDesc) -> i32>,
    pub make_texp: Option<unsafe extern "C" fn(*mut HSD_MObj, *mut HSD_TObj, *mut *mut HSD_TExp) -> *mut HSD_TExp>,
    pub setup_tev: Option<unsafe extern "C" fn(*mut HSD_MObj, *mut HSD_TObj, u32)>,
    pub unset: Option<unsafe extern "C" fn(*mut HSD_MObj, u32)>,
    pub tevdesc_tmpl: HSD_TevDesc,
    pub texp_tmpl: HSD_TECnst,
}

/// `it_mobj` (the C static initialiser). `it_mobj_static_init` must run once before use.
pub static mut it_mobj: it_MObjInfo = unsafe { core::mem::zeroed() };

/// Sets the static initial values of `it_mobj` (the C data initialiser). Idempotent.
pub unsafe fn it_mobj_static_init() {
    melee_core::ct!("it_mobj_static_init");
    let m = &raw mut it_mobj;
    (*m).parent = core::mem::zeroed();
    (*m).parent.head.info_init = Some(it_80277D08);
    (*m).setup = None;
    (*m).load = None;
    (*m).make_texp = None;
    (*m).setup_tev = None;
    (*m).unset = None;
    let mut t: HSD_TevDesc = core::mem::zeroed();
    t.next = null_mut();
    t.flags = 1;
    t.stage = 0;
    t.coord = 0xFF;
    t.map = 0xFF;
    t.color = 0xFF;
    // u.tevconf: the GX TEV fields in declaration order (GX_TEV_ADD ... GX_TEV_KASEL_1).
    let c = &mut t.u.tevconf;
    c.clr_op = GX_TEV_ADD as _;
    c.clr_a = GX_CC_CPREV as _;
    c.clr_b = GX_CC_ZERO as _;
    c.clr_c = GX_CC_ZERO as _;
    c.clr_d = GX_CC_ZERO as _;
    c.clr_scale = GX_CS_SCALE_1 as _;
    c.clr_bias = GX_TB_ZERO as _;
    c.clr_clamp = 1 as _;
    c.clr_out_reg = GX_TEVPREV as _;
    c.alpha_op = GX_TEV_ADD as _;
    c.alpha_a = GX_CA_ZERO as _;
    c.alpha_b = GX_CA_ZERO as _;
    c.alpha_c = GX_CA_ZERO as _;
    c.alpha_d = GX_CA_APREV as _;
    c.alpha_scale = GX_CS_SCALE_1 as _;
    c.alpha_bias = GX_TB_ZERO as _;
    c.alpha_clamp = 0 as _;
    c.alpha_out_reg = GX_TEVPREV as _;
    c.mode = GX_TC_LINEAR as _;
    c.ras_swap = GX_TEV_SWAP0 as _;
    c.tex_swap = GX_TEV_SWAP0 as _;
    c.kcsel = GX_TEV_KCSEL_1 as _;
    c.kasel = GX_TEV_KASEL_1 as _;
    (*m).tevdesc_tmpl = t;
    let mut tc: HSD_TECnst = core::mem::zeroed();
    tc.r#type = HSD_TE_CNST as _;
    tc.next = null_mut();
    tc.val = null_mut();
    tc.comp = HSD_TE_RGB as _;
    tc.ctype = HSD_TE_U8 as _;
    tc.reg = 0xFF;
    tc.idx = 0xFF;
    (*m).texp_tmpl = tc;
}

#[inline(always)]
fn item_of(gobj: *mut HSD_GObj) -> *mut Item {
    unsafe { (*gobj).user_data as *mut Item }
}

/// `OSReport` of the C build's console; no console on PC, so the text is dropped.
fn os_report(_msg: &str) {}

/// `it_80277D08`: registers the `it_mobj` class under `hsdMObj`'s base class.
pub unsafe extern "C" fn it_80277D08() {
    melee_core::ct!("it_80277D08");
    let m = &raw mut it_mobj;
    ext::hsdInitClassInfo(
        m as *mut HSD_ClassInfo,
        &raw mut ext::hsdMObj as *mut HSD_ClassInfo,
        c"sysdolphin_base_library".as_ptr() as *mut i8,
        c"it_mobj".as_ptr() as *mut i8,
        core::mem::size_of::<HSD_MObjInfo>() as i32,
        core::mem::size_of::<HSD_MObj>() as i32,
    );
    (*m).parent.release = ext::hsdMObj.parent.release;
    (*m).parent.amnesia = ext::hsdMObj.parent.amnesia;
    (*m).setup = Some(fn_80277D8C);
    (*m).load = ext::hsdMObj.load;
    (*m).make_texp = ext::hsdMObj.make_texp;
}

/// `fn_80277D8C`: the material setup callback (TEV state, shadow/toon chaining, render mode).
pub unsafe extern "C" fn fn_80277D8C(mobj: *mut HSD_MObj, rendermode_arg: u32, unused_arg: u32) {
    melee_core::ct!("fn_80277D8C");
    let _ = unused_arg;
    let mut tobj2: *mut HSD_TObj;
    let mut sp38: HSD_TExp = core::mem::zeroed();
    let mut pe_desc: HSD_PEDesc = core::mem::zeroed();
    let item = item_of(ext::HSD_GObj_804D7814);
    let mut setup_render_arg: *mut HSD_PEDesc;
    let mut tobj1_ptr: *mut *mut HSD_TObj;
    let mut rendermode: u32;

    ext::HSD_StateInitTev();
    rendermode = (*mobj).rendermode;
    let mat = (*mobj).mat;
    ext::HSD_SetMaterialColor((*mat).ambient, (*mat).diffuse, (*mat).specular, (*mat).alpha);
    if rendermode & 8 != 0 {
        ext::HSD_SetMaterialShininess((*(*mobj).mat).shininess);
    }
    tobj1_ptr = null_mut();
    tobj2 = (*mobj).tobj;
    if rendermode & RENDER_SHADOW != 0 && !ext::tobj_shadows.is_null() {
        tobj1_ptr = &raw mut tobj2;
        while !(*tobj1_ptr).is_null() {
            tobj1_ptr = &raw mut (**tobj1_ptr).next;
        }
        *tobj1_ptr = ext::tobj_shadows;
    }
    if rendermode & RENDER_TOON != 0 && !ext::tobj_toon.is_null() && !(*ext::tobj_toon).imagedesc.is_null() {
        (*ext::tobj_toon).next = tobj2;
        tobj2 = ext::tobj_toon;
    }
    ext::HSD_TObjSetup(tobj2);
    ext::HSD_TObjSetupTextureCoordGen(tobj2);
    let info = (*mobj).parent.class_info as *mut HSD_MObjInfo;
    ((*info).setup_tev.expect("setup_tev"))(mobj, tobj2, rendermode);
    it_80278108(item, mobj, it_80277F90(item, mobj, &raw mut sp38));
    if (*item).x5C9 != 0xFF {
        rendermode |= RENDER_BLENDING;
    }
    if (*item).xDCF_flag.anon_0.b4() != 0 && (*item).xDCF_flag.anon_0.b5() == 0 {
        rendermode |= RENDER_NO_ZUPDATE;
    }
    if (*item).xDCF_flag.anon_0.b5() != 0 && (*item).x5C9 == 0xFF {
        pe_desc.flags = 0x38;
        pe_desc.dst_alpha = 0;
        pe_desc.r#type = 0;
        pe_desc.src_factor = 4;
        setup_render_arg = &raw mut pe_desc;
        pe_desc.dst_factor = 5;
        pe_desc.logic_op = 0xF;
        pe_desc.z_comp = 3;
        pe_desc.alpha_comp0 = 7;
        pe_desc.ref0 = 0;
        pe_desc.alpha_op = 0;
        pe_desc.alpha_comp1 = 7;
        pe_desc.ref1 = 0;
    } else {
        setup_render_arg = (*mobj).pe;
    }
    ext::HSD_SetupRenderModeWithCustomPE(rendermode, setup_render_arg);
    if !tobj1_ptr.is_null() {
        *tobj1_ptr = null_mut();
    }
}

/// `it_80277F90`: for an item with a light overlay (and no `xDCF.b4`), fills `arg2` with a color constant
/// and a TEV stage; otherwise returns NULL.
pub unsafe extern "C" fn it_80277F90(item: *mut Item, mobj: *mut HSD_MObj, arg2: *mut HSD_TExp) -> *mut HSD_TExp {
    melee_core::ct!("it_80277F90");
    let mut desc: HSD_TevDesc;
    let info = &raw mut it_mobj;
    let reg: i32;
    let mut chk: i32;

    let ov = &raw mut (*item).x548_colorOverlay;
    if (*item).xDCF_flag.anon_0.b4() == 0 && (*ov).x7C_flag2() != 0 && (*ov).x7C_light_enable() != 0 {
        (*arg2).cnst = (*info).texp_tmpl;
        reg = ext::lbGetFreeColorRegister(0, mobj, null_mut());
        if reg == -1 {
            os_report("can't find free color register!\n");
            panic!("HSD_ASSERT(0xDD, 0)");
        }
        (*arg2).cnst.reg = reg as u8;
        (*arg2).cnst.val = &raw mut (*ov).x50_light_color as *mut c_void;
        ext::HSD_TExpSetReg(arg2);
        desc = (*info).tevdesc_tmpl;
        desc.stage = ext::HSD_StateAssignTev() as u32;
        desc.color = 2;
        desc.u.tevconf.clr_a = GX_CC_ZERO as _;
        desc.u.tevconf.clr_b = ext::lb_8000CC8C(reg) as _;
        desc.u.tevconf.clr_c = GX_CC_RASA as _;
        chk = 0;
        desc.u.tevconf.clr_d = chk as _;
        if reg < 4 {
            chk = 1;
        }
        if chk != 0 {
            desc.u.tevconf.kcsel = ext::lb_8000CCA4(reg) as _;
        }
        ext::HSD_SetupTevStage(&raw mut desc);
        return arg2;
    }
    null_mut()
}

/// `it_80278108`: the item's color overlay / body color / shadow-ratio TEV stages.
pub unsafe extern "C" fn it_80278108(item: *mut Item, mobj: *mut HSD_MObj, texp: *mut HSD_TExp) {
    melee_core::ct!("it_80278108");
    let mut sp168: _GXColor = core::mem::zeroed();
    let mut spFC: HSD_TECnst = core::mem::zeroed();
    let mut sp90: HSD_TECnst = core::mem::zeroed();
    let mut sp1C: HSD_TevDesc = core::mem::zeroed();
    let mut color: _GXColor = core::mem::zeroed();
    let mut chk1: bool = false;
    let mut var_r0: i32;
    let reg1: i32;
    let reg2: i32;
    let var_r3: i32;
    let info = &raw mut it_mobj;
    let mut overlay: *mut ColorOverlay;

    if (*item).xDCF_flag.anon_0.b5() != 0 {
        return;
    }
    overlay = &raw mut (*item).x548_colorOverlay;
    if (*item).xDCF_flag.anon_0.b3() != 0 {
        if !(*item).owner.is_null() && (*item).xDC8_word.flags.x13() != 0 {
            overlay = ext::ftCo_800C0674((*item).owner);
        }
    }
    if (*item).xDCF_flag.anon_0.b4() != 0 {
        chk1 = true;
        sp168 = (*item).xBC8.x0_unk;
    } else if (*item).x5C8 != 0 {
        if (*overlay).x7C_color_enable() != 0 {
            let temp_alpha: u32;
            let inv_alpha: i32;
            let item_color: *mut _GXColor = &raw mut (*item).xBC4;
            let color_hex: *mut _GXColor = &raw mut (*overlay).x2C_hex;
            let mut temp_r8: i32;
            let temp_r7: i32;

            temp_alpha = (((0xFF - (*item_color).a as i32) * (0xFF - (*color_hex).a as i32)) / 255) as u32;
            if temp_alpha as i32 == 0xFF {
                sp168 = (*overlay).x2C_hex;
            } else {
                inv_alpha = 0xFF - temp_alpha as i32;
                temp_r8 = (*item_color).r as i32;
                temp_r8 += ((*color_hex).a as i32 * ((*color_hex).r as i32 - temp_r8)) / 255;
                temp_r7 = temp_r8 * 0xFF;
                sp168.r = (temp_r7 / inv_alpha) as u8;
                if sp168.r != 0 {
                    sp168.a = (temp_r7 / sp168.r as i32) as u8;
                } else {
                    sp168.a = (((inv_alpha - temp_r8) * 0xFF) / 255) as u8;
                }
                {
                    let temp_r8_3 = (*item_color).g as i32;
                    sp168.g = ((((temp_r8_3 + (((*color_hex).a as i32) * ((*color_hex).g as i32 - temp_r8_3)) / 255)) * 0xFF) / inv_alpha) as u8;
                }
                {
                    let temp_r6 = (*item_color).b as i32;
                    sp168.b = ((((temp_r6 + (((*color_hex).a as i32) * ((*color_hex).b as i32 - temp_r6)) / 255)) * 0xFF) / inv_alpha) as u8;
                }
            }
            chk1 = true;
        } else {
            chk1 = true;
            sp168 = (*item).xBC4;
        }
    } else if (*overlay).x7C_color_enable() != 0 {
        chk1 = true;
        sp168 = (*overlay).x2C_hex;
    }
    if chk1 {
        spFC = (*info).texp_tmpl;
        reg1 = ext::lbGetFreeColorRegister(0, mobj, texp);
        if reg1 == -1 {
            os_report("can't find free color register!\n");
            panic!("HSD_ASSERT(0x144, 0)");
        }
        spFC.reg = reg1 as u8;
        spFC.val = &raw mut sp168 as *mut c_void;
        ext::HSD_TExpSetReg(&raw mut spFC as *mut HSD_TExp);
        spFC.next = texp;
        var_r0 = if reg1 < 4 { 1 } else { 0 };
        var_r3 = if var_r0 != 0 { 4 } else { 0 };
        reg2 = ext::lbGetFreeColorRegister(var_r3, mobj, &raw mut spFC as *mut HSD_TExp);
        if reg2 == -1 {
            os_report("can't find free color ratio register!\n");
            panic!("HSD_ASSERT(0x152, 0)");
        }
        if (*item).x5C9 != 0xFF {
            sp90 = (*info).texp_tmpl;
            sp90.reg = reg2 as u8;
            sp90.comp = 5 as _;
            sp90.idx = 3;
            sp90.val = &raw mut (*item).x5C9 as *mut c_void;
            spFC.next = &raw mut sp90 as *mut HSD_TExp;
        } else {
            spFC.next = null_mut();
        }
        spFC.reg = reg2 as u8;
        color.r = sp168.a;
        color.g = sp168.a;
        color.b = sp168.a;
        color.a = 0;
        spFC.val = &raw mut color as *mut c_void;
        ext::HSD_TExpSetReg(&raw mut spFC as *mut HSD_TExp);
        sp1C = (*info).tevdesc_tmpl;
        sp1C.stage = ext::HSD_StateAssignTev() as u32;
        sp1C.u.tevconf.clr_b = ext::lb_8000CC8C(reg1) as _;
        sp1C.u.tevconf.clr_c = ext::lb_8000CC8C(reg2) as _;
        var_r0 = if reg1 < 4 { 1 } else { 0 };
        if var_r0 != 0 {
            sp1C.u.tevconf.kcsel = ext::lb_8000CCA4(reg1) as _;
        } else {
            var_r0 = if reg2 < 4 { 1 } else { 0 };
            if var_r0 != 0 {
                sp1C.u.tevconf.kcsel = ext::lb_8000CCA4(reg2) as _;
            }
        }
        if (*item).x5C9 != 0xFF {
            sp1C.u.tevconf.alpha_d = ext::lb_8000CD90(reg2) as _;
            var_r0 = if reg2 < 4 { 1 } else { 0 };
            if var_r0 != 0 {
                sp1C.u.tevconf.kasel = ext::lb_8000CDA8(reg2) as _;
            }
        }
        ext::HSD_SetupTevStage(&raw mut sp1C);
    }
}

/// `it_80278574`: walks the jobj tree under `gobj` and sets every material's diffuse color to `arg1`.
pub unsafe extern "C" fn it_80278574(gobj: *mut HSD_GObj, arg1: *mut _GXColor) {
    melee_core::ct!("it_80278574");
    let mut var_r31: *mut HSD_JObj = (*gobj).hsd_obj as *mut HSD_JObj;
    while !var_r31.is_null() {
        let mut var_r3: *mut HSD_DObj = ext::HSD_JObjGetDObj(var_r31);
        while !var_r3.is_null() {
            let temp_r4 = (*var_r3).mobj;
            if !temp_r4.is_null() {
                let temp_r4_2 = (*temp_r4).mat;
                if !temp_r4_2.is_null() {
                    (*temp_r4_2).diffuse = *arg1;
                }
            }
            var_r3 = (*var_r3).next;
        }
        if !(*var_r31).child.is_null() {
            var_r31 = (*var_r31).child;
        } else if !(*var_r31).next.is_null() {
            var_r31 = (*var_r31).next;
        } else {
            // loop_25: climb to the nearest ancestor that has a following sibling.
            loop {
                let parent = (*var_r31).parent;
                if parent.is_null() {
                    var_r31 = null_mut();
                    break;
                }
                if !(*parent).next.is_null() {
                    var_r31 = (*parent).next;
                    break;
                }
                var_r31 = parent;
            }
        }
    }
}
