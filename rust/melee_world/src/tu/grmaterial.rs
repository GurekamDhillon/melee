//! src/melee/gr/grmaterial.c: stage material helpers (flag walkers over a JObj tree's MObjs, the
//! stage MObj class `gr_mobj` whose setup adds the colour-overlay / alpha TEV stage, the item
//! spawn wrappers and the colour-overlay script commands).
//!
//! Pointer-to-int cast: `HSD_IDInsertToTable(NULL, (HSD_IDKey) jobj, jobj)` is a `uintptr_t` cast
//! (native `usize` here; the GC build truncates to 32 bits).
//! No float arithmetic (the one int-to-float `(f32) val` is exact).
//! State-oracle checked against the C (tests/gro_grmaterial.rs, gen/oracle/gr_oracle.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use super::grinl::{assert_, HSD_JObjGetChild, HSD_JObjGetNext, HSD_JObjGetParent};
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_types::*;

/// Callees in TUs that are not ported yet. An unset hook panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use core::ffi::c_void;
    use melee_types::*;
    pub static mut HSD_IDINSERTTOTABLE: Option<fn(*mut HSD_IDTable, HSD_IDKey, *mut c_void)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_IDInsertToTable(table: *mut HSD_IDTable, id: HSD_IDKey, data: *mut c_void) {
        match unsafe { HSD_IDINSERTTOTABLE } { Some(f) => f(table, id, data), None => { unsafe { let _ = melee_runtime::tu::id::HSD_IDInsertToTable(table as _, id as _, data as _); } return; } }
    }
    hook!(HSD_JOBJGETDOBJ, HSD_JObjGetDObj, (jobj: *mut HSD_JObj) -> *mut HSD_DObj);
    // rp3 HSD_MObjClearFlags
    hook_or!(HSD_MOBJCLEARFLAGS, HSD_MObjClearFlags, (mobj: *mut HSD_MObj, flags: u32), |mobj: *mut HSD_MObj, flags: u32| unsafe { melee_runtime::tu::mobj::HSD_MObjClearFlags(mobj, flags); });
    // rp3-end

    // rp3 HSD_MObjSetFlags
    hook_or!(HSD_MOBJSETFLAGS, HSD_MObjSetFlags, (mobj: *mut HSD_MObj, flags: u32), |mobj: *mut HSD_MObj, flags: u32| unsafe { melee_runtime::tu::mobj::HSD_MObjSetFlags(mobj, flags); });
    // rp3-end

    hook!(HSD_SETMATERIALCOLOR, HSD_SetMaterialColor, (ambient: GXColor, diffuse: GXColor, specular: GXColor, alpha: f32));
    hook!(HSD_SETMATERIALSHININESS, HSD_SetMaterialShininess, (shininess: f32));
    hook!(HSD_SETUPRENDERMODEWITHCUSTOMPE, HSD_SetupRenderModeWithCustomPE, (rendermode: u32, pe: *mut HSD_PEDesc));
    hook!(HSD_SETUPTEVSTAGE, HSD_SetupTevStage, (a0: *mut HSD_TevDesc));
    hook!(HSD_STATEASSIGNTEV, HSD_StateAssignTev, () -> i32);
    hook!(HSD_STATEINITTEV, HSD_StateInitTev, ());
    pub static mut HSD_TEXPSETREG: Option<fn(*mut HSD_TExp)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_TExpSetReg(texp: *mut HSD_TExp) {
        match unsafe { HSD_TEXPSETREG } { Some(f) => f(texp), None => { unsafe { let _ = melee_runtime::tu::texp::HSD_TExpSetReg(texp as _); } return; } }
    }
    pub static mut HSD_TEXPSETUPTEV: Option<fn(*mut HSD_TExpTevDesc, *mut HSD_TExp)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_TExpSetupTev(a0: *mut HSD_TExpTevDesc, a1: *mut HSD_TExp) {
        match unsafe { HSD_TEXPSETUPTEV } { Some(f) => f(a0, a1), None => { unsafe { let _ = melee_runtime::tu::texp::HSD_TExpSetupTev(a0 as _, a1 as _); } return; } }
    }
    pub static mut HSD_TOBJSETUP: Option<fn(*mut HSD_TObj)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_TObjSetup(tobj: *mut HSD_TObj) {
        match unsafe { HSD_TOBJSETUP } { Some(f) => f(tobj), None => { unsafe { let _ = melee_runtime::tu::tobj::HSD_TObjSetup(tobj as _); } return; } }
    }
    pub static mut HSD_TOBJSETUPTEXTURECOORDGEN: Option<fn(*mut HSD_TObj)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_TObjSetupTextureCoordGen(tobj: *mut HSD_TObj) {
        match unsafe { HSD_TOBJSETUPTEXTURECOORDGEN } { Some(f) => f(tobj), None => { unsafe { let _ = melee_runtime::tu::tobj::HSD_TObjSetupTextureCoordGen(tobj as _); } return; } }
    }
    pub static mut HSD_TOBJSETUPVOLATILETEV: Option<fn(*mut HSD_TObj, u32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_TObjSetupVolatileTev(tobj: *mut HSD_TObj, rendermode: u32) {
        match unsafe { HSD_TOBJSETUPVOLATILETEV } { Some(f) => f(tobj, rendermode), None => { unsafe { let _ = melee_runtime::tu::tobj::HSD_TObjSetupVolatileTev(tobj as _, rendermode as _); } return; } }
    }
    hook!(ITEM_80268E5C, Item_80268E5C, (gobj: *mut HSD_GObj, msid: enum_t, flags: Item_StateChangeFlags));
    hook!(ITEM_8026A8EC, Item_8026A8EC, (gobj: *mut HSD_GObj));
    hook!(OSREPORT, OSReport, (a0: *mut i8));
    pub static mut HSDCHANGECLASS: Option<fn(*mut c_void, *mut c_void) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn hsdChangeClass(object: *mut c_void, class_info: *mut c_void) -> i32 {
        match unsafe { HSDCHANGECLASS } { Some(f) => f(object, class_info), None => { return unsafe { melee_runtime::tu::class::hsdChangeClass(object as _, class_info as _) as _ }; } }
    }
    pub static mut HSDINITCLASSINFO: Option<fn(*mut HSD_ClassInfo, *mut HSD_ClassInfo, *mut i8, *mut i8, s32, s32)> = None;
    #[allow(non_snake_case)]
    pub fn hsdInitClassInfo(class_info: *mut HSD_ClassInfo, parent_info: *mut HSD_ClassInfo, base_class_library: *mut i8, ty: *mut i8, info_size: s32, class_size: s32) {
        match unsafe { HSDINITCLASSINFO } { Some(f) => f(class_info, parent_info, base_class_library, ty, info_size, class_size), None => { unsafe { let _ = melee_runtime::tu::class::hsdInitClassInfo(class_info as _, parent_info as _, base_class_library as _, ty as _, info_size as _, class_size as _); } return; } }
    }
    hook!(IT_802725D4, it_802725D4, (a0: *mut HSD_GObj));
    hook!(IT_802756D0, it_802756D0, (a0: *mut HSD_GObj));
    hook!(IT_802756E0, it_802756E0, (a0: *mut HSD_GObj));
    hook!(IT_802E6AEC, it_802E6AEC, (a0: *mut Ground, a1: i32, a2: i32, a3: *mut HSD_JObj, a4: *mut Vec3, a5: i32, a6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, a7: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec3, *mut HSD_GObj, f32)>, a8: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>) -> *mut HSD_GObj);
    pub static mut LBGETFREEALPHAREGISTER: Option<fn(s32, *mut HSD_MObj, *mut HSD_TExp) -> s32> = None;
    #[allow(non_snake_case)]
    pub fn lbGetFreeAlphaRegister(i0: s32, a1: *mut HSD_MObj, a2: *mut HSD_TExp) -> s32 {
        match unsafe { LBGETFREEALPHAREGISTER } { Some(f) => f(i0, a1, a2), None => { return unsafe { melee_runtime::tu::lb_00B0::lbGetFreeAlphaRegister(i0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut LBGETFREECOLORREGISTER: Option<fn(s32, *mut HSD_MObj, *mut HSD_TExp) -> s32> = None;
    #[allow(non_snake_case)]
    pub fn lbGetFreeColorRegister(i0: s32, a1: *mut HSD_MObj, a2: *mut HSD_TExp) -> s32 {
        match unsafe { LBGETFREECOLORREGISTER } { Some(f) => f(i0, a1, a2), None => { return unsafe { melee_runtime::tu::lb_00B0::lbGetFreeColorRegister(i0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut LB_8000CC8C: Option<fn(s32) -> s32> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000CC8C(i: s32) -> s32 {
        match unsafe { LB_8000CC8C } { Some(f) => f(i), None => { return unsafe { melee_runtime::tu::lb_00B0::lb_8000CC8C(i as _) as _ }; } }
    }
    pub static mut LB_8000CCA4: Option<fn(s32) -> s32> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000CCA4(i: s32) -> s32 {
        match unsafe { LB_8000CCA4 } { Some(f) => f(i), None => { return unsafe { melee_runtime::tu::lb_00B0::lb_8000CCA4(i as _) as _ }; } }
    }
    pub static mut LB_8000CD90: Option<fn(s32) -> s32> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000CD90(i: s32) -> s32 {
        match unsafe { LB_8000CD90 } { Some(f) => f(i), None => { return unsafe { melee_runtime::tu::lb_00B0::lb_8000CD90(i as _) as _ }; } }
    }
    hook!(LB_80014258, lb_80014258, (gobj: *mut HSD_GObj, arg1: *mut c_void, cmd: FtCmd2) -> i32);
    hook!(LB_80014498, lb_80014498, (a0: *mut ColorOverlay));
    // Data owned by other TUs.
    /// `HSD_GObj* HSD_GObj_804D7814` (the gobj being drawn).
    pub static mut HSD_GObj_804D7814: *mut HSD_GObj = core::ptr::null_mut();
    /// `HSD_TObj* tobj_shadows` / `tobj_toon` (baselib/tobj.c).
    pub static mut tobj_shadows: *mut HSD_TObj = core::ptr::null_mut();
    pub static mut tobj_toon: *mut HSD_TObj = core::ptr::null_mut();
    /// `HSD_MObjInfo hsdMObj` (baselib/mobj.c).
    pub static mut hsdMObj: HSD_MObjInfo = unsafe { core::mem::zeroed() };
}

/// `struct grMaterial_MObjInfo` (file-local): the stage MObj class info.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grMaterial_MObjInfo {
    /*   +0 */ pub parent: HSD_ClassInfo,
    /*  +3C */ pub setup: HSD_MObjSetupFunc,
    /*  +40 */ pub load: Option<unsafe extern "C" fn(*mut HSD_MObj, *mut HSD_MObjDesc) -> i32>,
    /*  +44 */ pub make_texp: Option<unsafe extern "C" fn(*mut HSD_MObj, *mut HSD_TObj, *mut *mut HSD_TExp) -> *mut HSD_TExp>,
    /*  +48 */ pub setup_tev: Option<unsafe extern "C" fn(*mut HSD_MObj, *mut HSD_TObj, u32)>,
    /*  +4C */ pub unset: Option<unsafe extern "C" fn(*mut HSD_MObj, u32)>,
    /*  +50 */ pub tevdesc_tmpl: HSD_TevDesc,
    /*  +C4 */ pub texp_tmpl: HSD_TECnst,
}

const GX_DISABLE: u8 = 0;

const fn tevdesc_tmpl() -> HSD_TevDesc {
    let mut d: HSD_TevDesc = unsafe { core::mem::zeroed() };
    d.next = null_mut();
    d.flags = 1;
    d.stage = 0;
    d.coord = 0xFF;
    d.map = 0xFF;
    d.color = 0xFF;
    d.u = _HSD_TevDesc_u {
        tevconf: HSD_TevConf {
            clr_op: GX_TEV_ADD,
            clr_a: GX_CC_CPREV,
            clr_b: GX_CC_ZERO,
            clr_c: GX_CC_ZERO,
            clr_d: GX_CC_ZERO,
            clr_scale: GX_CS_SCALE_1,
            clr_bias: GX_TB_ZERO,
            clr_clamp: GX_DISABLE,
            clr_out_reg: GX_TEVPREV,
            alpha_op: GX_TEV_ADD,
            alpha_a: GX_CA_ZERO,
            alpha_b: GX_CA_ZERO,
            alpha_c: GX_CA_ZERO,
            alpha_d: GX_CA_APREV,
            alpha_scale: GX_CS_SCALE_1,
            alpha_bias: GX_TB_ZERO,
            alpha_clamp: GX_DISABLE,
            alpha_out_reg: GX_TEVPREV,
            mode: GX_TC_LINEAR,
            ras_swap: GX_TEV_SWAP0,
            tex_swap: GX_TEV_SWAP0,
            kcsel: GX_TEV_KCSEL_1,
            kasel: GX_TEV_KASEL_1,
        },
    };
    d
}

/// `static struct grMaterial_MObjInfo grMaterial_803E0A20` (3E0A20).
pub static mut grMaterial_803E0A20: grMaterial_MObjInfo = {
    let mut parent: HSD_ClassInfo = unsafe { core::mem::zeroed() };
    parent.head.info_init = Some(grMaterial_801C8E74);
    let mut texp: HSD_TECnst = unsafe { core::mem::zeroed() };
    texp.r#type = HSD_TE_CNST;
    texp.comp = HSD_TE_RGB;
    texp.ctype = HSD_TE_U8;
    texp.reg = 0xFF;
    texp.idx = 0xFF;
    grMaterial_MObjInfo {
        parent,
        setup: None,
        load: None,
        make_texp: None,
        setup_tev: None,
        unset: None,
        tevdesc_tmpl: tevdesc_tmpl(),
        texp_tmpl: texp,
    }
};

/// `static ItCmd grMaterial_804D456C[1]` (4D456C).
pub static mut grMaterial_804D456C: [ItCmd; 1] = [Some(grMaterial_801C9470)];

pub unsafe fn grMaterial_801C87D0(jobj: *mut HSD_JObj, flags: u32) {
    if !jobj.is_null() {
        let mut dobj = ext::HSD_JObjGetDObj(jobj);
        while !dobj.is_null() {
            let mobj = if !dobj.is_null() { (*dobj).mobj } else { null_mut() };
            if !mobj.is_null() {
                ext::HSD_MObjSetFlags(mobj, flags);
            }
            dobj = if !dobj.is_null() { (*dobj).next } else { null_mut() };
        }
    }
}

pub unsafe fn grMaterial_801C8858(jobj: *mut HSD_JObj, arg1: u32) {
    let mut jobj = jobj;
    if jobj.is_null() {
        return;
    }
    grMaterial_801C87D0(jobj, arg1);
    jobj = if jobj.is_null() { null_mut() } else { (*jobj).child };
    while !jobj.is_null() {
        grMaterial_801C8858(jobj, arg1);
        jobj = if jobj.is_null() { null_mut() } else { (*jobj).next };
    }
}

/// `static void grMaterial_801C897C(HSD_JObj*, u32)`.
pub unsafe fn grMaterial_801C897C(jobj: *mut HSD_JObj, flags: u32) {
    if !jobj.is_null() {
        let mut dobj = ext::HSD_JObjGetDObj(jobj);
        while !dobj.is_null() {
            let mobj = if !dobj.is_null() { (*dobj).mobj } else { null_mut() };
            if !mobj.is_null() {
                ext::HSD_MObjClearFlags(mobj, flags);
            }
            dobj = if !dobj.is_null() { (*dobj).next } else { null_mut() };
        }
    }
}

pub unsafe fn grMaterial_801C8A04(jobj: *mut HSD_JObj, arg1: u32) {
    let mut jobj = jobj;
    if jobj.is_null() {
        return;
    }
    grMaterial_801C897C(jobj, arg1);
    jobj = if jobj.is_null() { null_mut() } else { (*jobj).child };
    while !jobj.is_null() {
        grMaterial_801C8A04(jobj, arg1);
        jobj = if jobj.is_null() { null_mut() } else { (*jobj).next };
    }
}

pub unsafe fn grMaterial_801C8B28(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    let mut jobj = jobj;
    if !jobj.is_null() {
        while !HSD_JObjGetParent(jobj).is_null() {
            jobj = HSD_JObjGetParent(jobj);
        }
    }
    jobj
}

/// The tail of `grMaterial_801C8B68`'s two inlined copies: `var_r3` is set when `arg1 == 0` and
/// the jobj has flag 0x4000 (the C's `!arg1;` statement has no effect).
#[inline(always)]
unsafe fn flag_4000(jobj: *mut HSD_JObj, arg1: i32) -> i32 {
    let mut var_r3 = 0;
    if arg1 != 0 {
    } else {
        let var_r0 = if (*jobj).flags & 0x4000 != 0 { 1 } else { 0 };
        if var_r0 != 0 {
            var_r3 = 1;
        }
    }
    var_r3
}

pub unsafe fn grMaterial_801C8B68(jobj: *mut HSD_JObj, arg1: i32) {
    let mut jobj = jobj;
    if !jobj.is_null() {
        let var_r3 = flag_4000(jobj, arg1);
        if var_r3 != 0 {
            ext::HSD_IDInsertToTable(null_mut(), jobj as HSD_IDKey, jobj as *mut c_void);
        }
        if (*jobj).flags & 0x1000 == 0 {
            jobj = HSD_JObjGetChild(jobj);
            while !jobj.is_null() {
                let mut var_r30 = jobj;
                let var_r3 = flag_4000(jobj, arg1);
                if var_r3 != 0 {
                    ext::HSD_IDInsertToTable(null_mut(), var_r30 as HSD_IDKey, var_r30 as *mut c_void);
                }
                if (*jobj).flags & 0x1000 == 0 {
                    var_r30 = HSD_JObjGetChild(jobj);
                    while !var_r30.is_null() {
                        grMaterial_801C8B68(var_r30, arg1);
                        var_r30 = HSD_JObjGetNext(var_r30);
                    }
                }
                jobj = HSD_JObjGetNext(jobj);
            }
        }
    }
}

pub unsafe fn grMaterial_801C8CDC(gobj: *mut HSD_GObj) {
    ext::Item_8026A8EC(gobj);
}

pub unsafe fn grMaterial_801C8CFC(
    arg0: i32,
    arg1: i32,
    arg2: *mut Ground,
    arg3: *mut HSD_JObj,
    arg4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>,
    arg5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec3, *mut HSD_GObj, f32)>,
    arg6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>,
) -> *mut HSD_GObj {
    ext::it_802E6AEC(arg2, arg0, arg1, arg3, null_mut(), 0, arg4, arg5, arg6)
}

pub unsafe fn grMaterial_801C8D44(
    arg0: i32,
    arg1: i32,
    arg2: *mut Ground,
    arg3: *mut Vec3,
    arg4: i32,
    arg5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>,
    arg6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec3, *mut HSD_GObj, f32)>,
    arg7: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>,
) -> *mut HSD_GObj {
    ext::it_802E6AEC(arg2, arg0, arg1, null_mut(), arg3, arg4, arg5, arg6, arg7)
}

pub unsafe fn grMaterial_801C8D98(gobj: *mut HSD_GObj, id: i32) {
    ext::it_802725D4(gobj);
    ext::Item_80268E5C(gobj, id, 2);
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn grMaterial_801C8DE0(gobj: *mut HSD_GObj, arg8: f32, arg9: f32, arg_a: f32, arg_b: f32, arg_c: f32, arg_d: f32, arg_e: f32) {
    let it = (*gobj).user_data as *mut Item;
    let hurt: *mut HurtCapsule = (*it).xACC_itemHurtbox.as_mut_ptr();
    (*hurt).a_offset.x = arg8;
    (*hurt).a_offset.y = arg9;
    (*hurt).a_offset.z = arg_a;
    (*hurt).b_offset.x = arg_b;
    (*hurt).b_offset.y = arg_c;
    (*hurt).b_offset.z = arg_d;
    (*hurt).scale = arg_e;
}

pub unsafe fn grMaterial_801C8E08(gobj: *mut HSD_GObj) {
    ext::it_802756E0(gobj);
}

pub unsafe fn grMaterial_801C8E28(gobj: *mut HSD_GObj) {
    ext::it_802756D0(gobj);
}

pub unsafe fn grMaterial_801C8E48(gobj: *mut HSD_GObj) -> bool {
    let it = (*gobj).user_data as *mut Item;
    if !(*it).x524_cmd.anon_8.u.is_null() {
        return false;
    }
    true
}

pub unsafe fn grMaterial_801C8E68(gobj: *mut HSD_GObj, ground_or_air: GroundOrAir) {
    let it = (*gobj).user_data as *mut Item;
    (*it).ground_or_air = ground_or_air;
}

/// The class-info initialiser of `gr_mobj` (the `info_init` of `grMaterial_803E0A20`).
pub unsafe extern "C" fn grMaterial_801C8E74() {
    ext::hsdInitClassInfo(
        addr_of_mut!(grMaterial_803E0A20) as *mut HSD_ClassInfo,
        addr_of_mut!(ext::hsdMObj) as *mut HSD_ClassInfo,
        b"sysdolphin_base_library\0".as_ptr() as *mut i8,
        b"gr_mobj\0".as_ptr() as *mut i8,
        core::mem::size_of::<HSD_MObjInfo>() as s32,
        core::mem::size_of::<HSD_MObj>() as s32,
    );
    grMaterial_803E0A20.parent.release = ext::hsdMObj.parent.release;
    grMaterial_803E0A20.parent.amnesia = ext::hsdMObj.parent.amnesia;
    grMaterial_803E0A20.setup = Some(fn_801C8EF8);
    grMaterial_803E0A20.load = ext::hsdMObj.load;
    grMaterial_803E0A20.make_texp = ext::hsdMObj.make_texp;
}

#[inline(always)]
unsafe fn grMaterial_SetTExpReg(cnst: *mut HSD_TECnst) {
    ext::HSD_TExpSetReg(cnst as *mut HSD_TExp);
}

#[inline(always)]
unsafe fn grMaterial_GetFreeColorReg(cnst: *mut HSD_TECnst, mobj: *mut HSD_MObj, start: s32) -> s32 {
    ext::lbGetFreeColorRegister(start, mobj, cnst as *mut HSD_TExp)
}

#[inline(always)]
unsafe fn grMaterial_GetKColorSel(reg: s32) -> GXTevKColorSel {
    ext::lb_8000CCA4(reg) as GXTevKColorSel
}

/// `static inline void fn_801C8EF8_inline(Ground*, HSD_MObj*)`.
unsafe fn fn_801C8EF8_inline(gp: *mut Ground, mobj: *mut HSD_MObj) {
    // The C locals are zero-initialised (-ftrivial-auto-var-init=zero / gwtool: uninitialised
    // locals are 0), which also zeroes `color.a` and the struct padding.
    let mut cnst: HSD_TECnst = core::mem::zeroed();
    let mut tevdesc: HSD_TevDesc = core::mem::zeroed();
    let mut color: GXColor = core::mem::zeroed();

    if (*gp).color_overlay.x7C_color_enable() != 0 || (*gp).x10_flags.b6() != 0 {
        cnst = grMaterial_803E0A20.texp_tmpl;
        tevdesc = grMaterial_803E0A20.tevdesc_tmpl;
        tevdesc.stage = ext::HSD_StateAssignTev() as u32;
        if (*gp).color_overlay.x7C_color_enable() != 0 {
            let mut reg1_lt4: i32;
            let mut temp: i32;

            let reg1 = ext::lbGetFreeColorRegister(0, mobj, null_mut());
            if reg1 == -1 {
                let mut msg = *b"can't find free color register!\n\0";
                ext::OSReport(msg.as_mut_ptr() as *mut i8);
                assert_(false, 0x7A, "0");
            }
            reg1_lt4 = 1;
            cnst.comp = reg1_lt4 as _;
            temp = 0;
            cnst.ctype = temp as _;
            cnst.reg = reg1 as u8;
            cnst.val = addr_of_mut!((*gp).color_overlay.x2C_hex) as *mut c_void;
            grMaterial_SetTExpReg(&mut cnst);
            if reg1 < 4 {
            } else {
                reg1_lt4 = temp;
            }
            if reg1_lt4 != 0 {
                temp = 4;
            } else {
                temp = 0;
            }
            let reg2 = grMaterial_GetFreeColorReg(&mut cnst, mobj, temp);
            if reg2 == -1 {
                let mut msg = *b"can't find free color register!\n\0";
                ext::OSReport(msg.as_mut_ptr() as *mut i8);
                assert_(false, 0x88, "0");
            }
            temp = 1;
            cnst.comp = temp as _;
            let reg1_lt4_for_kcsel = 0;
            cnst.ctype = reg1_lt4_for_kcsel as _;
            cnst.reg = reg2 as u8;
            color.r = (*gp).color_overlay.x2C_hex.a;
            color.g = (*gp).color_overlay.x2C_hex.a;
            color.b = (*gp).color_overlay.x2C_hex.a;
            cnst.val = &mut color as *mut GXColor as *mut c_void;
            grMaterial_SetTExpReg(&mut cnst);
            tevdesc.u.tevconf.clr_b = ext::lb_8000CC8C(reg1) as _;
            tevdesc.u.tevconf.clr_c = ext::lb_8000CC8C(reg2) as _;
            if reg1 < 4 {
            } else {
                temp = reg1_lt4_for_kcsel;
            }
            if temp != 0 {
                tevdesc.u.tevconf.kcsel = grMaterial_GetKColorSel(reg1);
            } else {
                temp = if reg2 < 4 { 1 } else { 0 };
                if temp != 0 {
                    tevdesc.u.tevconf.kcsel = ext::lb_8000CCA4(reg2) as GXTevKColorSel;
                }
            }
        }
        if (*gp).x10_flags.b6() != 0 {
            let alpha_reg = ext::lbGetFreeAlphaRegister(0, mobj, null_mut());
            if alpha_reg == -1 {
                let mut msg = *b"can't find free alpha register!\n\0";
                ext::OSReport(msg.as_mut_ptr() as *mut i8);
                assert_(false, 0xA7, "0");
            }
            cnst.comp = 5;
            cnst.ctype = 3;
            cnst.reg = alpha_reg as u8;
            cnst.val = addr_of_mut!((*gp).xC0) as *mut c_void;
            grMaterial_SetTExpReg(&mut cnst);
            tevdesc.u.tevconf.alpha_b = GX_CA_APREV;
            tevdesc.u.tevconf.alpha_c = ext::lb_8000CD90(alpha_reg) as _;
            tevdesc.u.tevconf.alpha_d = GX_CA_ZERO;
        }
        ext::HSD_SetupTevStage(&mut tevdesc);
    }
    let _ = (cnst, tevdesc, color);
}

/// `static void fn_801C8EF8(HSD_MObj*, u32)`: the `gr_mobj` setup.
pub unsafe extern "C" fn fn_801C8EF8(mobj: *mut HSD_MObj, _rendermode: u32) {
    ext::HSD_StateInitTev();
    let mobj_rendermode = (*mobj).rendermode;
    ext::HSD_SetMaterialColor((*(*mobj).mat).ambient, (*(*mobj).mat).diffuse, (*(*mobj).mat).specular, (*(*mobj).mat).alpha);
    if mobj_rendermode & RENDER_SPECULAR as u32 != 0 {
        ext::HSD_SetMaterialShininess((*(*mobj).mat).shininess);
    }
    let mut cur_tobj: *mut *mut HSD_TObj = null_mut();
    let mut tobj: *mut HSD_TObj = (*mobj).tobj;
    if mobj_rendermode & RENDER_SHADOW as u32 != 0 && !ext::tobj_shadows.is_null() {
        cur_tobj = &mut tobj;
        while !(*cur_tobj).is_null() {
            cur_tobj = &mut (**cur_tobj).next;
        }
        *cur_tobj = ext::tobj_shadows;
    }
    if mobj_rendermode & RENDER_TOON as u32 != 0 && !ext::tobj_toon.is_null() && !(*ext::tobj_toon).imagedesc.is_null() {
        (*ext::tobj_toon).next = tobj;
        tobj = ext::tobj_toon;
    }
    ext::HSD_TObjSetup(tobj);
    ext::HSD_TObjSetupTextureCoordGen(tobj);
    assert_(!(*mobj).tevdesc.is_null(), 0xF3, "mobj->tevdesc");
    ext::HSD_TExpSetupTev((*mobj).tevdesc, (*mobj).texp);
    ext::HSD_TObjSetupVolatileTev(tobj, mobj_rendermode);

    let gp = (*ext::HSD_GObj_804D7814).user_data as *mut Ground;
    fn_801C8EF8_inline(gp, mobj);
    ext::HSD_SetupRenderModeWithCustomPE(mobj_rendermode, (*mobj).pe);
    if !cur_tobj.is_null() {
        *cur_tobj = null_mut();
    }
}

/// `if (jobj->flags & 0x4020) cond = 0; else cond = 1;` then `hsdChangeClass` on each MObj.
unsafe fn change_class_dobjs(jobj: *mut HSD_JObj) {
    let cond = if (*jobj).flags & 0x4020 != 0 { 0 } else { 1 };
    if cond != 0 {
        let mut dobj = ext::HSD_JObjGetDObj(jobj);
        while !dobj.is_null() {
            let mobj = if !dobj.is_null() { (*dobj).mobj } else { null_mut() };
            if !mobj.is_null() {
                ext::hsdChangeClass(mobj as *mut c_void, addr_of_mut!(grMaterial_803E0A20) as *mut c_void);
            }
            dobj = if !dobj.is_null() { (*dobj).next } else { null_mut() };
        }
    }
}

pub unsafe fn grMaterial_801C92C0(jobj: *mut HSD_JObj) {
    let mut jobj = jobj;
    if jobj.is_null() {
        return;
    }
    change_class_dobjs(jobj);
    jobj = if jobj.is_null() { null_mut() } else { (*jobj).child };
    while !jobj.is_null() {
        let mut grandchild = jobj;
        if !jobj.is_null() {
            change_class_dobjs(grandchild);
            grandchild = if jobj.is_null() { null_mut() } else { (*jobj).child };
            while !grandchild.is_null() {
                grMaterial_801C92C0(grandchild);
                grandchild = if grandchild.is_null() { null_mut() } else { (*grandchild).next };
            }
        }
        jobj = if jobj.is_null() { null_mut() } else { (*jobj).next };
    }
}

pub unsafe extern "C" fn grMaterial_801C9470(gobj: *mut HSD_GObj, cmd: *mut CommandInfo) {
    grMaterial_801C9490(gobj, cmd);
}

pub unsafe fn grMaterial_801C9490(gobj: *mut HSD_GObj, cmd: *mut CommandInfo) {
    let gp = (*gobj).user_data as *mut Ground;
    let val: u32 = ((*((*cmd).anon_8.ptr[0] as *const u16) as u32) >> 2) & 0xFF;
    (*gp).xC0 = val as f32;
    (*gp).x10_flags.set_b6(1);
}

pub unsafe fn grMaterial_801C94D8(obj: *mut c_void) {
    let hsd_obj = obj as *mut HSD_JObj;
    if !hsd_obj.is_null() {
        change_class_dobjs(hsd_obj);
        let mut var_r30_2 = if hsd_obj.is_null() { null_mut() } else { (*hsd_obj).child };
        while !var_r30_2.is_null() {
            grMaterial_801C92C0(var_r30_2);
            var_r30_2 = if var_r30_2.is_null() { null_mut() } else { (*var_r30_2).next };
        }
    }
}

pub unsafe fn grMaterial_801C95C4(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    ext::lb_80014498(addr_of_mut!((*gp).color_overlay));
    (*gp).x10_flags.set_b4(1);
}

pub unsafe fn grMaterial_801C9604(gobj: *mut HSD_GObj, script: *mut c_void, arg2: bool) {
    let gp = (*gobj).user_data as *mut Ground;
    let co = addr_of_mut!((*gp).color_overlay);
    (*co).x4_pri = arg2 as i32;
    (*co).x8_ptr1 = script as *mut ColorOverlay_x8_t;
    (*co).x0_timer = 0;
    (*co).xC_loop = 0;
    (*co).set_x7C_flag2(0);
    (*co).set_x7C_color_enable(0);
    (*gp).x10_flags.set_b4(0);
    grMaterial_801C9698(gobj);
}

pub unsafe extern "C" fn fn_801C9664(gobj: *mut HSD_GObj, cmd: *mut CommandInfo, arg2: i32) {
    let idx = (arg2 - 21) as usize;
    (grMaterial_804D456C[idx].unwrap())(gobj, cmd);
}

pub unsafe fn grMaterial_801C9698(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    if ext::lb_80014258(gobj, addr_of_mut!((*gp).color_overlay) as *mut c_void, Some(fn_801C9664)) != 0 {
        (*gp).x10_flags.set_b4(1);
    }
}
