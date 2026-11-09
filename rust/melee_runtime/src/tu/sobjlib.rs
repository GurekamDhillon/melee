//! Native port of src/sysdolphin/baselib/sobjlib.c.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_variables, unused_assignments, static_mut_refs, clippy::all)]
use melee_types::*;
use ext::*;
macro_rules! OSReport { ($fmt:expr $(,)?) => { ext::OSReport($fmt, core::ptr::null_mut()) }; ($fmt:expr, $arg:expr $(,)?) => { ext::OSReport($fmt, $arg) }; }
pub unsafe extern "C" fn HSD_GObjGetHSDObj(
    mut gobj: *mut HSD_GObj,
) -> *mut ::core::ffi::c_void {
    melee_core::ct!("HSD_GObjGetHSDObj");
    return (*gobj).hsd_obj;
}
pub const PROJ_ORTHO: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
#[inline]
pub unsafe extern "C" fn GXEnd() { melee_core::ct!("GXEnd");}
pub static mut HSD_SObjLib_804D7960: u8 = 0;
pub static mut HSD_SObjLib_8040C3A0: [GObjFunc; 1] = unsafe {
    [
        ::core::mem::transmute::<
            *mut ::core::ffi::c_void,
            GObjFunc,
        >(
            ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut HSD_SObj) -> ()>,
                *mut ::core::ffi::c_void,
            >(Some(HSD_SObjLib_803A4740 as unsafe extern "C" fn(*mut HSD_SObj) -> ())),
        ),
    ]
};
pub static mut HSD_SObjLib_8040C3A4: GObjFuncs = GObjFuncs {
        next: ::core::ptr::null::<GObjFuncs>() as *mut GObjFuncs,
        size: 1 as u8,
        funcs: &raw const HSD_SObjLib_8040C3A0 as *mut GObjFunc,
};
pub static mut HSD_SObjLib_804D10E0: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub unsafe extern "C" fn HSD_SObjLib_803A44A4() {
    melee_core::ct!("HSD_SObjLib_803A44A4");
    HSD_ObjAllocInit(
        &raw mut HSD_SObjLib_804D10E0,
        ::core::mem::size_of::<HSD_SObj>() as u32,
        4 as u32,
    );
}
pub unsafe extern "C" fn HSD_SObjLib_803A44D4(
    mut gobj: *mut HSD_GObj,
    mut sobj: *mut HSD_SObj,
    mut priority: u8,
) {
    melee_core::ct!("HSD_SObjLib_803A44D4");
    let mut cur: *mut HSD_SObj = ::core::ptr::null_mut::<HSD_SObj>();
    let mut next: *mut HSD_SObj = ::core::ptr::null_mut::<HSD_SObj>();
    let _: [::core::ffi::c_uchar; 16] = [0; 16];
    cur = (*sobj).prev;
    if !cur.is_null() || !(*sobj).next.is_null() {
        if !cur.is_null() {
            (*cur).next = (*sobj).next;
        } else {
            HSD_GObjObject_80390ADC(gobj);
            HSD_GObjObject_80390A70(
                gobj,
                HSD_SObjLib_804D7960,
                (*sobj).next as *mut ::core::ffi::c_void,
            );
            (*(*sobj).next).prev = ::core::ptr::null_mut::<HSD_SObj>();
        }
        if !(*sobj).next.is_null() {
            (*(*sobj).next).prev = (*sobj).prev;
        } else {
            (*(*sobj).prev).next = ::core::ptr::null_mut::<HSD_SObj>();
        }
        (*sobj).next = ::core::ptr::null_mut::<HSD_SObj>();
        (*sobj).prev = ::core::ptr::null_mut::<HSD_SObj>();
    } else if (*gobj).hsd_obj == sobj as *mut ::core::ffi::c_void {
        HSD_GObjObject_80390ADC(gobj);
    }
    cur = (*gobj).hsd_obj as *mut HSD_SObj;
    if cur.is_null() {
        HSD_GObjObject_80390ADC(gobj);
        HSD_GObjObject_80390A70(
            gobj,
            HSD_SObjLib_804D7960,
            sobj as *mut ::core::ffi::c_void,
        );
    } else {
        cur = (*gobj).hsd_obj as *mut HSD_SObj;
        next = cur;
        while !(*next).next.is_null()
            && (*next).x44 as ::core::ffi::c_int <= priority as ::core::ffi::c_int
        {
            next = (*next).next;
        }
        if (*next).next.is_null()
            && (*next).x44 as ::core::ffi::c_int <= priority as ::core::ffi::c_int
        {
            (*next).next = sobj;
            (*sobj).prev = next;
        } else if next == cur {
            (*sobj).next = cur;
            let ref mut fresh0 = (*((*gobj).hsd_obj as *mut HSD_SObj)).prev;
            *fresh0 = sobj;
            HSD_GObjObject_80390ADC(gobj);
            HSD_GObjObject_80390A70(
                gobj,
                HSD_SObjLib_804D7960,
                sobj as *mut ::core::ffi::c_void,
            );
        } else {
            (*(*next).prev).next = sobj;
            (*sobj).prev = (*next).prev;
            (*next).prev = sobj;
            (*sobj).next = next;
        }
    }
    (*sobj).x44 = priority;
}
pub unsafe extern "C" fn HSD_SObjLib_803A466C(mut sobj: *mut HSD_SObj) {
    melee_core::ct!("HSD_SObjLib_803A466C");
    let _: [::core::ffi::c_uchar; 8] = [0; 8];
    if sobj.is_null() {
        return;
    }
    if !(*sobj).prev.is_null() || !(*sobj).next.is_null() {
        if !(*sobj).prev.is_null() {
            (*(*sobj).prev).next = (*sobj).next;
        } else {
            HSD_GObjObject_80390ADC((*sobj).gobj);
            HSD_GObjObject_80390A70(
                (*sobj).gobj,
                HSD_SObjLib_804D7960,
                (*sobj).next as *mut ::core::ffi::c_void,
            );
            (*(*sobj).next).prev = ::core::ptr::null_mut::<HSD_SObj>();
        }
        if !(*sobj).next.is_null() {
            (*(*sobj).next).prev = (*sobj).prev;
        } else {
            (*(*sobj).prev).next = ::core::ptr::null_mut::<HSD_SObj>();
        }
        (*sobj).next = ::core::ptr::null_mut::<HSD_SObj>();
        (*sobj).prev = ::core::ptr::null_mut::<HSD_SObj>();
    } else if (*(*sobj).gobj).hsd_obj == sobj as *mut ::core::ffi::c_void {
        HSD_GObjObject_80390ADC((*sobj).gobj);
    }
    HSD_ObjFree(&raw mut HSD_SObjLib_804D10E0, sobj as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn HSD_SObjLib_803A4740(mut sobj: *mut HSD_SObj) {
    melee_core::ct!("HSD_SObjLib_803A4740");
    let mut next: *mut HSD_SObj = ::core::ptr::null_mut::<HSD_SObj>();
    while !sobj.is_null() {
        next = (*sobj).next;
        HSD_SObjLib_803A466C(sobj);
        sobj = next;
    }
}
pub unsafe extern "C" fn HSD_SObjLib_803A477C(
    mut gobj: *mut HSD_GObj,
    mut desc: *mut HSD_SObjDesc,
    mut wrap_s: GXTexWrapMode,
    mut wrap_t: GXTexWrapMode,
    mut priority: u8,
    mut use_secondary: u8,
) -> *mut HSD_SObj {
    melee_core::ct!("HSD_SObjLib_803A477C");
    let mut image: *mut HSD_ImageDesc = ::core::ptr::null_mut::<HSD_ImageDesc>();
    let mut tlut: *mut HSD_Tlut = ::core::ptr::null_mut::<HSD_Tlut>();
    let mut image2: *mut HSD_ImageDesc = ::core::ptr::null_mut::<HSD_ImageDesc>();
    let mut sobj: *mut HSD_SObj = ::core::ptr::null_mut::<HSD_SObj>();
    let mut inv_width: f32 = 0.;
    let mut inv_height: f32 = 0.;
    if use_secondary != 0 {
        image = (*desc).image;
        tlut = (*desc).tlut as *mut HSD_Tlut;
        image2 = (*(desc as *mut HSD_SObjDesc2)).image2;
    } else {
        image = (*desc).image;
        image2 = ::core::ptr::null_mut::<HSD_ImageDesc>();
        tlut = (*desc).tlut as *mut HSD_Tlut;
    }
    sobj = HSD_ObjAlloc(&raw mut HSD_SObjLib_804D10E0) as *mut HSD_SObj;
    if !sobj.is_null() {} else {
        panic!("HSD_SObjLib assertion");
    };
    if !tlut.is_null() {
        GXInitTlutObj(
            &raw mut (*sobj).x70_tlutobj,
            (*tlut).lut,
            (*tlut).fmt,
            (*tlut).n_entries,
        );
        GXInitTexObjCI(
            &raw mut (*sobj).x50_texobj,
            (*image).image_ptr as *mut ::core::ffi::c_void,
            (*image).width,
            (*image).height,
            (*image).format,
            wrap_s,
            wrap_t,
            (*image).mipmap as u8,
            (*tlut).tlut_name,
        );
    } else {
        GXInitTexObj(
            &raw mut (*sobj).x50_texobj,
            (*image).image_ptr as *mut ::core::ffi::c_void,
            (*image).width,
            (*image).height,
            (*image).format,
            wrap_s,
            wrap_t,
            (*image).mipmap as u8,
        );
    }
    if use_secondary != 0 {
        GXInitTexObj(
            &raw mut (*sobj).x7C_texobj,
            (*image2).image_ptr as *mut ::core::ffi::c_void,
            (*image2).width,
            (*image2).height,
            (*image2).format,
            wrap_s,
            wrap_t,
            (*image2).mipmap as u8,
        );
    }
    (*sobj).x0 = ::core::ptr::null_mut::<::core::ffi::c_void>();
    (*sobj).prev = ::core::ptr::null_mut::<HSD_SObj>();
    (*sobj).next = ::core::ptr::null_mut::<HSD_SObj>();
    (*sobj).x14 = 0.0f32 as f32;
    (*sobj).x10 = 0.0f32 as f32;
    (*sobj).x18 = 0.0f32 as f32;
    (*sobj).x20 = 1.0f32 as f32;
    (*sobj).x1C = 1.0f32 as f32;
    (*sobj).x3C_u.x0.x3F = 0xff as u8;
    (*sobj).x3C_u.x0.x3E = 0xff as u8;
    (*sobj).x3C_u.x0.x3D = 0xff as u8;
    (*sobj).x3C_u.x0.x3C = 0xff as u8;
    (*sobj).x38_u.x0.x3B = 0xff as u8;
    (*sobj).x38_u.x0.x3A = 0xff as u8;
    (*sobj).x38_u.x0.x39 = 0xff as u8;
    (*sobj).x38_u.x0.x38 = 0xff as u8;
    (*sobj).x40 = 0 as u32;
    (*sobj).x48 = 0 as u32;
    (*sobj).x4C_callback = None;
    (*sobj).gobj = gobj;
    if use_secondary != 0 {
        (*sobj).x40 |= 4 as u32;
    }
    (*sobj).x34 = (*image).width;
    (*sobj).x36 = (*image).height;
    inv_width = 1.0f32 / GXGetTexObjWidth(&raw mut (*sobj).x50_texobj) as f32;
    inv_height = 1.0f32 / GXGetTexObjHeight(&raw mut (*sobj).x50_texobj) as f32;
    (*sobj).x24 = 0.0f32 as f32;
    (*sobj).x28 = 0.0f32 as f32;
    (*sobj).x2C = (*sobj).x34 as f32 * inv_width;
    (*sobj).x30 = (*sobj).x36 as f32 * inv_height;
    HSD_SObjLib_803A44D4(gobj, sobj, priority);
    return sobj;
}
pub unsafe extern "C" fn HSD_SObjLib_803A49E0(
    mut gobj: *mut HSD_GObj,
    mut unused: ::core::ffi::c_int,
) {
    melee_core::ct!("HSD_SObjLib_803A49E0");
    let mut sobj: *mut HSD_SObj = ::core::ptr::null_mut::<HSD_SObj>();
    let _: [::core::ffi::c_uchar; 8] = [0; 8];
    sobj = (*gobj).hsd_obj as *mut HSD_SObj;
    while !sobj.is_null() {
        if HSD_SObjLib_804D7960 as ::core::ffi::c_int
            != (*gobj).obj_kind as ::core::ffi::c_int
        {
            OSReport!(
                b"BadOBJ for SOBJ-displayfunc\n\0" as *const u8
                    as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            return;
        }
        if (*sobj).x4C_callback.is_some() {
            (*sobj).x4C_callback.expect("non-null function pointer")(sobj);
        }
        HSD_SObjLib_803A4A68(sobj);
        sobj = (*sobj).next;
    }
}
#[inline]
pub unsafe extern "C" fn discard_color(mut color: GXColor) -> GXColor {
    melee_core::ct!("discard_color");
    return color;
}
#[cfg_attr(fptrace, melee_fptrace::trace)]
pub unsafe extern "C" fn HSD_SObjLib_803A4A68(mut sobj: *mut HSD_SObj) {
    melee_core::ct!("HSD_SObjLib_803A4A68");
    let mut x_cos: f32 = 0.;
    let mut origin_x: f32 = 0.;
    let mut origin_y: f32 = 0.;
    let mut angle: f32 = 0.;
    let mut sin_half: f32 = 0.;
    let mut cos_val: f32 = 0.;
    let mut cos_half: f32 = 0.;
    let mut width: f32 = 0.;
    let mut height: f32 = 0.;
    let mut x_sin: f32 = 0.;
    let mut y_sin: f32 = 0.;
    let mut y_cos: f32 = 0.;
    let mut tex_s: f32 = 0.;
    let mut tex_t: f32 = 0.;
    let mut tex_s_max: f32 = 0.;
    let mut tex_t_max: f32 = 0.;
    let mut left_x: f32 = 0.;
    let mut left_y: f32 = 0.;
    let mut right_x: f32 = 0.;
    let mut right_y: f32 = 0.;
    let mut obj_width: u16 = 0;
    let mut obj_height: u16 = 0;
    let mut tex_fmt: u8 = 0;
    if (*sobj).x40 & 1 as u32 != 0 {
        return;
    }
    obj_width = (*sobj).x34;
    obj_height = (*sobj).x36;
    tex_fmt = GXGetTexObjFmt(&raw mut (*sobj).x50_texobj) as u8;
    if (*sobj).x40 & 2 as u32 != 0 {
        origin_x = (*sobj).x10;
        origin_y = (*sobj).x14;
    } else {
        origin_x = obj_width as f32 * (*sobj).x1C / 2.0f32 + (*sobj).x10;
        origin_y = obj_height as f32 * (*sobj).x20 / 2.0f32 + (*sobj).x14;
    }
    GXClearVtxDesc();
    if (*sobj).x40 & 4 as u32 != 0 {
        GXSetZTexture(GX_ZT_REPLACE, GX_TF_Z24X8, 0 as u32);
        if (*sobj).x40 & 8 as u32 != 0 {
            HSD_StateSetZMode(
                1 as ::core::ffi::c_int,
                GX_LESS as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        } else {
            HSD_StateSetZMode(
                1 as ::core::ffi::c_int,
                GX_LESS as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
        }
        HSD_StateSetZCompLoc(0 as ::core::ffi::c_int);
        GXSetCullMode(GX_CULL_BACK);
        GXSetNumTexGens(1 as u8);
        GXSetNumTevStages(2 as u8);
    } else {
        HSD_StateSetZMode(
            1 as ::core::ffi::c_int,
            GX_ALWAYS as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        GXSetCullMode(GX_CULL_BACK);
        if (*sobj).x40 & 0x10 as u32 != 0 {
            GXSetNumTexGens(2 as u8);
            GXSetNumTevStages(4 as u8);
        } else {
            GXSetNumTexGens(1 as u8);
            GXSetNumTevStages(1 as u8);
        }
    }
    if (tex_fmt as ::core::ffi::c_int - GX_TF_C4 as ::core::ffi::c_int) as u8
        as ::core::ffi::c_uint <= 1 as ::core::ffi::c_uint
    {
        GXLoadTlut(
            &raw mut (*sobj).x70_tlutobj,
            GX_TLUT0 as ::core::ffi::c_int as u32,
        );
    }
    if (*sobj).x40 & 0x10 as u32 == 0 {
        GXLoadTexObj(&raw mut (*sobj).x50_texobj, GX_TEXMAP0);
    }
    GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XY, GX_F32, 0 as u8);
    GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
    GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_TEX0, GX_TEX_ST, GX_F32, 0 as u8);
    GXSetVtxDesc(GX_VA_TEX0, GX_DIRECT);
    GXSetColorUpdate(1 as GXBool);
    GXSetAlphaUpdate(0 as GXBool);
    if (*sobj).x40 & 0x10 as u32 != 0 {
        GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD1, GX_TEXMAP1, GX_COLOR_NULL);
        GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_TEXC, GX_CC_KONST, GX_CC_C0);
        GXSetTevColorOp(
            GX_TEVSTAGE0,
            GX_TEV_ADD,
            GX_TB_ZERO,
            GX_CS_SCALE_1,
            GX_FALSE,
            GX_TEVPREV,
        );
        GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_ZERO, GX_CA_TEXA, GX_CA_KONST, GX_CA_A0);
        GXSetTevAlphaOp(
            GX_TEVSTAGE0,
            GX_TEV_SUB,
            GX_TB_ZERO,
            GX_CS_SCALE_1,
            GX_FALSE,
            GX_TEVPREV,
        );
        GXSetTevKColorSel(GX_TEVSTAGE0, GX_TEV_KCSEL_K0);
        GXSetTevKAlphaSel(GX_TEVSTAGE0, GX_TEV_KASEL_K0_A);
        GXSetTevSwapMode(GX_TEVSTAGE0, GX_TEV_SWAP0, GX_TEV_SWAP0);
        GXSetTevOrder(GX_TEVSTAGE1, GX_TEXCOORD1, GX_TEXMAP2, GX_COLOR_NULL);
        GXSetTevColorIn(GX_TEVSTAGE1, GX_CC_ZERO, GX_CC_TEXC, GX_CC_KONST, GX_CC_CPREV);
        GXSetTevColorOp(
            GX_TEVSTAGE1,
            GX_TEV_ADD,
            GX_TB_ZERO,
            GX_CS_SCALE_2,
            GX_FALSE,
            GX_TEVPREV,
        );
        GXSetTevAlphaIn(GX_TEVSTAGE1, GX_CA_ZERO, GX_CA_TEXA, GX_CA_KONST, GX_CA_APREV);
        GXSetTevAlphaOp(
            GX_TEVSTAGE1,
            GX_TEV_SUB,
            GX_TB_ZERO,
            GX_CS_SCALE_1,
            GX_FALSE,
            GX_TEVPREV,
        );
        GXSetTevKColorSel(GX_TEVSTAGE1, GX_TEV_KCSEL_K1);
        GXSetTevKAlphaSel(GX_TEVSTAGE1, GX_TEV_KASEL_K1_A);
        GXSetTevSwapMode(GX_TEVSTAGE1, GX_TEV_SWAP0, GX_TEV_SWAP0);
        GXSetTevOrder(GX_TEVSTAGE2, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR_NULL);
        GXSetTevColorIn(GX_TEVSTAGE2, GX_CC_ZERO, GX_CC_TEXC, GX_CC_ONE, GX_CC_CPREV);
        GXSetTevColorOp(
            GX_TEVSTAGE2,
            GX_TEV_ADD,
            GX_TB_ZERO,
            GX_CS_SCALE_1,
            GX_TRUE,
            GX_TEVPREV,
        );
        GXSetTevAlphaIn(GX_TEVSTAGE2, GX_CA_TEXA, GX_CA_ZERO, GX_CA_ZERO, GX_CA_APREV);
        GXSetTevAlphaOp(
            GX_TEVSTAGE2,
            GX_TEV_ADD,
            GX_TB_ZERO,
            GX_CS_SCALE_1,
            GX_TRUE,
            GX_TEVPREV,
        );
        GXSetTevSwapMode(GX_TEVSTAGE2, GX_TEV_SWAP0, GX_TEV_SWAP0);
        GXSetTevOrder(GX_TEVSTAGE3, GX_TEXCOORD_NULL, GX_TEXMAP_NULL, GX_COLOR_NULL);
        GXSetTevColorIn(GX_TEVSTAGE3, GX_CC_APREV, GX_CC_CPREV, GX_CC_KONST, GX_CC_ZERO);
        GXSetTevColorOp(
            GX_TEVSTAGE3,
            GX_TEV_ADD,
            GX_TB_ZERO,
            GX_CS_SCALE_1,
            GX_TRUE,
            GX_TEVPREV,
        );
        GXSetTevAlphaIn(GX_TEVSTAGE3, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO);
        GXSetTevAlphaOp(
            GX_TEVSTAGE3,
            GX_TEV_ADD,
            GX_TB_ZERO,
            GX_CS_SCALE_1,
            GX_TRUE,
            GX_TEVPREV,
        );
        GXSetTevSwapMode(GX_TEVSTAGE3, GX_TEV_SWAP0, GX_TEV_SWAP0);
        GXSetTevKColorSel(GX_TEVSTAGE3, GX_TEV_KCSEL_K2);
        GXSetAlphaCompare(GX_ALWAYS, 0 as u8, GX_AOP_OR, GX_ALWAYS, 0 as u8);
        GXSetTevColorS10(
            GX_TEVREG0,
            _GXColorS10 {
                r: -(90 as ::core::ffi::c_int) as s16,
                g: 0 as s16,
                b: -(114 as ::core::ffi::c_int) as s16,
                a: 135 as s16,
            },
        );
        GXSetTevKColor(
            GX_KCOLOR0,
            _GXColor {
                r: 0 as u8,
                g: 0 as u8,
                b: 0xe2 as u8,
                a: 0x58 as u8,
            },
        );
        GXSetTevKColor(
            GX_KCOLOR1,
            _GXColor {
                r: 0xb3 as u8,
                g: 0 as u8,
                b: 0 as u8,
                a: 0xb6 as u8,
            },
        );
        GXSetTevKColor(
            GX_KCOLOR2,
            _GXColor {
                r: 0xff as u8,
                g: 0 as u8,
                b: 0xff as u8,
                a: 0x80 as u8,
            },
        );
        GXSetTevSwapModeTable(
            GX_TEV_SWAP0,
            GX_CH_RED,
            GX_CH_GREEN,
            GX_CH_BLUE,
            GX_CH_ALPHA,
        );
        GXSetBlendMode(GX_BM_NONE, GX_BL_ONE, GX_BL_ZERO, GX_LO_CLEAR);
    } else {
        GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        if (*sobj).x40 & 4 as u32 != 0 {
            GXLoadTexObj(&raw mut (*sobj).x7C_texobj, GX_TEXMAP1);
            GXSetTevOrder(GX_TEVSTAGE1, GX_TEXCOORD0, GX_TEXMAP1, GX_COLOR0A0);
        }
        GXSetAlphaCompare(GX_ALWAYS, 0 as u8, GX_AOP_OR, GX_ALWAYS, 0 as u8);
        if (*sobj).x40 & 4 as u32 != 0 {
            GXSetTevColor(GX_TEVREG0, (*sobj).x3C_u.x3C_color);
            GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_TEXC, GX_CC_C0, GX_CC_ZERO);
            GXSetTevColorOp(
                GX_TEVSTAGE0,
                GX_TEV_ADD,
                GX_TB_ZERO,
                GX_CS_SCALE_1,
                GX_TRUE,
                GX_TEVPREV,
            );
            GXSetTevColorIn(
                GX_TEVSTAGE1,
                GX_CC_ZERO,
                GX_CC_ZERO,
                GX_CC_ZERO,
                GX_CC_CPREV,
            );
            GXSetTevColorOp(
                GX_TEVSTAGE1,
                GX_TEV_ADD,
                GX_TB_ZERO,
                GX_CS_SCALE_1,
                GX_TRUE,
                GX_TEVPREV,
            );
            GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO, GX_CA_A0);
            GXSetTevAlphaOp(
                GX_TEVSTAGE0,
                GX_TEV_ADD,
                GX_TB_ZERO,
                GX_CS_SCALE_1,
                GX_TRUE,
                GX_TEVPREV,
            );
            GXSetTevAlphaIn(GX_TEVSTAGE1, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO, GX_CA_A0);
            GXSetTevAlphaOp(
                GX_TEVSTAGE1,
                GX_TEV_ADD,
                GX_TB_ZERO,
                GX_CS_SCALE_1,
                GX_TRUE,
                GX_TEVPREV,
            );
            GXSetBlendMode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR);
        } else {
            match tex_fmt as ::core::ffi::c_int {
                0 | 1 => {
                    GXSetTevColor(GX_TEVREG0, (*sobj).x38_u.x38_color);
                    GXSetTevColor(GX_TEVREG1, (*sobj).x3C_u.x3C_color);
                    GXSetTevColorIn(
                        GX_TEVSTAGE0,
                        GX_CC_C0,
                        GX_CC_C1,
                        GX_CC_TEXC,
                        GX_CC_ZERO,
                    );
                    GXSetTevColorOp(
                        GX_TEVSTAGE0,
                        GX_TEV_ADD,
                        GX_TB_ZERO,
                        GX_CS_SCALE_1,
                        GX_TRUE,
                        GX_TEVPREV,
                    );
                    GXSetTevAlphaIn(
                        GX_TEVSTAGE0,
                        GX_CA_ZERO,
                        GX_CA_ZERO,
                        GX_CA_ZERO,
                        GX_CA_TEXA,
                    );
                    GXSetTevAlphaOp(
                        GX_TEVSTAGE0,
                        GX_TEV_ADD,
                        GX_TB_ZERO,
                        GX_CS_SCALE_1,
                        GX_TRUE,
                        GX_TEVPREV,
                    );
                }
                2 | 3 => {
                    GXSetTevColor(GX_TEVREG0, (*sobj).x38_u.x38_color);
                    GXSetTevColor(GX_TEVREG1, (*sobj).x3C_u.x3C_color);
                    GXSetTevColorIn(
                        GX_TEVSTAGE0,
                        GX_CC_C0,
                        GX_CC_C1,
                        GX_CC_TEXC,
                        GX_CC_ZERO,
                    );
                    GXSetTevColorOp(
                        GX_TEVSTAGE0,
                        GX_TEV_ADD,
                        GX_TB_ZERO,
                        GX_CS_SCALE_1,
                        GX_TRUE,
                        GX_TEVPREV,
                    );
                    GXSetTevAlphaIn(
                        GX_TEVSTAGE0,
                        GX_CA_ZERO,
                        GX_CA_ZERO,
                        GX_CA_ZERO,
                        GX_CA_TEXA,
                    );
                    GXSetTevAlphaOp(
                        GX_TEVSTAGE0,
                        GX_TEV_ADD,
                        GX_TB_ZERO,
                        GX_CS_SCALE_1,
                        GX_TRUE,
                        GX_TEVPREV,
                    );
                }
                4 => {
                    GXSetTevColor(GX_TEVREG0, (*sobj).x3C_u.x3C_color);
                    GXSetTevColorIn(
                        GX_TEVSTAGE0,
                        GX_CC_ZERO,
                        GX_CC_TEXC,
                        GX_CC_C0,
                        GX_CC_ZERO,
                    );
                    GXSetTevColorOp(
                        GX_TEVSTAGE0,
                        GX_TEV_ADD,
                        GX_TB_ZERO,
                        GX_CS_SCALE_1,
                        GX_TRUE,
                        GX_TEVPREV,
                    );
                    GXSetTevAlphaIn(
                        GX_TEVSTAGE0,
                        GX_CA_ZERO,
                        GX_CA_ZERO,
                        GX_CA_ZERO,
                        GX_CA_A0,
                    );
                    GXSetTevAlphaOp(
                        GX_TEVSTAGE0,
                        GX_TEV_ADD,
                        GX_TB_ZERO,
                        GX_CS_SCALE_1,
                        GX_TRUE,
                        GX_TEVPREV,
                    );
                }
                5 | 6 | 8 | 9 | 10 | 14 => {
                    GXSetTevColor(GX_TEVREG0, (*sobj).x3C_u.x3C_color);
                    GXSetTevColorIn(
                        GX_TEVSTAGE0,
                        GX_CC_ZERO,
                        GX_CC_TEXC,
                        GX_CC_C0,
                        GX_CC_ZERO,
                    );
                    GXSetTevColorOp(
                        GX_TEVSTAGE0,
                        GX_TEV_ADD,
                        GX_TB_ZERO,
                        GX_CS_SCALE_1,
                        GX_TRUE,
                        GX_TEVPREV,
                    );
                    GXSetTevAlphaIn(
                        GX_TEVSTAGE0,
                        GX_CA_ZERO,
                        GX_CA_TEXA,
                        GX_CA_A0,
                        GX_CA_ZERO,
                    );
                    GXSetTevAlphaOp(
                        GX_TEVSTAGE0,
                        GX_TEV_ADD,
                        GX_TB_ZERO,
                        GX_CS_SCALE_1,
                        GX_TRUE,
                        GX_TEVPREV,
                    );
                }
                _ => {}
            }
            GXSetBlendMode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR);
        }
    }
    discard_color((*sobj).x38_u.x38_color);
    discard_color((*sobj).x3C_u.x3C_color);
    sin_half = (0.5f32 * crate::gekko::sinf((*sobj).x18 as ::core::ffi::c_float)) as f32;
    angle = (*sobj).x18;
    cos_val = crate::gekko::cosf(angle as ::core::ffi::c_float) as f32;
    cos_half = 0.5f32 * -cos_val;
    width = obj_width as f32 * (*sobj).x1C;
    height = obj_height as f32 * (*sobj).x20;
    x_sin = sin_half * width;
    x_cos = cos_half * width;
    y_sin = sin_half * height;
    y_cos = cos_half * height;
    GXBegin(GX_QUADS, GX_VTXFMT0, 4 as u16);
    left_y = origin_y + y_cos;
    right_x = origin_x + y_sin;
    right_y = origin_y - y_cos;
    GXPosition2f32(x_cos + right_x, -(left_y - x_sin));
    left_x = origin_x - y_sin;
    tex_t = (*sobj).x28;
    tex_s = (*sobj).x24;
    GXTexCoord2f32(tex_s, tex_t);
    GXPosition2f32(right_x - x_cos, -(x_sin + left_y));
    tex_s_max = (*sobj).x2C;
    GXTexCoord2f32(tex_s_max, tex_t);
    GXPosition2f32(left_x - x_cos, -(x_sin + right_y));
    tex_t_max = (*sobj).x30;
    GXTexCoord2f32(tex_s_max, tex_t_max);
    GXPosition2f32(x_cos + left_x, -(right_y - x_sin));
    GXTexCoord2f32(tex_s, tex_t_max);
    GXEnd();
    if (*sobj).x40 & 4 as u32 != 0 {
        GXSetZTexture(GX_ZT_DISABLE, GX_TF_Z24X8, 0 as u32);
    }
    HSD_StateInvalidate(-(1 as ::core::ffi::c_int));
    HSD_StateInitTev();
    HSD_ClearVtxDesc();
    HSD_StateSetZMode(
        1 as ::core::ffi::c_int,
        GX_LEQUAL as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
}
pub static mut HSD_SObjLib_804DEA98: f32 = 2.0f32;
pub static mut lbl_8040C418: HSD_Chan = HSD_Chan {
    next: ::core::ptr::null::<HSD_Chan>() as *mut HSD_Chan,
    chan: GX_COLOR0,
    flags: 0 as u32,
    amb_color: _GXColor {
        r: 0 as u8,
        g: 0,
        b: 0,
        a: 0,
    },
    mat_color: _GXColor {
        r: 0xff as u8,
        g: 0xff as u8,
        b: 0xff as u8,
        a: 0,
    },
    enable: 0 as GXBool,
    amb_src: GX_SRC_REG,
    mat_src: GX_SRC_REG,
    light_mask: GX_LIGHT_NULL,
    diff_fn: GX_DF_CLAMP,
    attn_fn: GX_AF_NONE,
    aobj: ::core::ptr::null::<HSD_AObj>() as *mut HSD_AObj,
};
pub static mut lbl_8040C448: HSD_Chan = HSD_Chan {
    next: ::core::ptr::null::<HSD_Chan>() as *mut HSD_Chan,
    chan: GX_ALPHA0,
    flags: 0 as u32,
    amb_color: _GXColor {
        r: 0 as u8,
        g: 0 as u8,
        b: 0 as u8,
        a: 0xff as u8,
    },
    mat_color: _GXColor {
        r: 0 as u8,
        g: 0 as u8,
        b: 0 as u8,
        a: 0xff as u8,
    },
    enable: 0 as GXBool,
    amb_src: GX_SRC_REG,
    mat_src: GX_SRC_REG,
    light_mask: GX_LIGHT_NULL,
    diff_fn: GX_DF_CLAMP,
    attn_fn: GX_AF_NONE,
    aobj: ::core::ptr::null::<HSD_AObj>() as *mut HSD_AObj,
};
pub unsafe extern "C" fn HSD_SObjLib_803A54EC(
    mut gobj: *mut HSD_GObj,
    mut unused: ::core::ffi::c_int,
) {
    melee_core::ct!("HSD_SObjLib_803A54EC");
    let mut viewmtx: Mtx = [[0.; 4]; 3];
    if HSD_CObjSetCurrent(HSD_GObjGetHSDObj(gobj) as *mut HSD_CObj) != 0 {
        GXSetCurrentMtx(0 as u32);
        HSD_CObjGetViewingMtx(HSD_CObjGetCurrent(), &raw mut viewmtx as *mut core::ffi::c_void);
        GXLoadPosMtxImm(&raw mut viewmtx as *mut core::ffi::c_void, 0 as u32);
        GXSetTexCoordGen2(
            GX_TEXCOORD0,
            GX_TG_MTX2x4,
            GX_TG_TEX0,
            GX_IDENTITY as ::core::ffi::c_int as u32,
            GX_FALSE,
            GX_PTIDENTITY as ::core::ffi::c_int as u32,
        );
        GXSetTexCoordGen2(
            GX_TEXCOORD1,
            GX_TG_MTX2x4,
            GX_TG_TEX0,
            GX_IDENTITY as ::core::ffi::c_int as u32,
            GX_FALSE,
            GX_PTIDENTITY as ::core::ffi::c_int as u32,
        );
        HSD_SetupChannel(&raw mut lbl_8040C418);
        HSD_SetupChannel(&raw mut lbl_8040C448);
        HSD_StateSetNumChans(1 as ::core::ffi::c_int);
        HSD_GObj_80390ED0(gobj, 1 as u32);
        HSD_CObjEndCurrent();
    } else {
        OSReport!(
            b"Out CameraDisp Range\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        );
    }
    HSD_StateInvalidate(-(1 as ::core::ffi::c_int));
    HSD_StateSetZMode(
        1 as ::core::ffi::c_int,
        3 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
}
pub unsafe extern "C" fn HSD_SObjLib_803A55DC(
    mut gobj: *mut HSD_GObj,
    mut width: u16,
    mut height: u16,
    mut priority: ::core::ffi::c_int,
) {
    melee_core::ct!("HSD_SObjLib_803A55DC");
    let mut cobj: *mut HSD_CObj = ::core::ptr::null_mut::<HSD_CObj>();
    let mut viewport: Scissor = _Scissor {
        left: 0,
        right: 0,
        top: 0,
        bottom: 0,
    };
    let mut scissor: Scissor = _Scissor {
        left: 0,
        right: 0,
        top: 0,
        bottom: 0,
    };
    let mut eye: Vec3 = Vec3 {
        x: 0 as ::core::ffi::c_int as f32,
        y: 0 as ::core::ffi::c_int as f32,
        z: 1 as ::core::ffi::c_int as f32,
    };
    let mut interest: Vec3 = Vec3 {
        x: 0 as ::core::ffi::c_int as f32,
        y: 0 as ::core::ffi::c_int as f32,
        z: 0 as ::core::ffi::c_int as f32,
    };
    let mut roll: f32 = 0.0f32;
    let mut near_val: f32 = roll;
    let mut far_val: f32 = *(&raw const HSD_SObjLib_804DEA98 as *const f32);
    let mut top: f32 = roll;
    let mut bottom: f32 = -(height as ::core::ffi::c_int) as f32;
    let mut left: f32 = roll;
    let mut right: f32 = width as f32;
    viewport.left = 0 as u16;
    viewport.right = width;
    viewport.top = 0 as u16;
    viewport.bottom = height;
    scissor.left = 0 as u16;
    scissor.right = width;
    scissor.top = 0 as u16;
    scissor.bottom = height;
    cobj = HSD_CObjAlloc();
    HSD_CObjSetProjectionType(cobj, PROJ_ORTHO as u32);
    HSD_CObjSetViewport(cobj, &raw mut viewport as *mut HSD_RectS16);
    HSD_CObjSetScissor(cobj, &raw mut scissor);
    HSD_CObjSetEyePosition(cobj, &raw mut eye);
    HSD_CObjSetInterest(cobj, &raw mut interest);
    HSD_CObjSetRoll(cobj, roll);
    HSD_CObjSetNear(cobj, near_val);
    HSD_CObjSetFar(cobj, far_val);
    HSD_CObjSetOrtho(cobj, top, bottom, left, right);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_CameraKind, cobj as *mut ::core::ffi::c_void);
    GObj_SetupGXLinkMax(
        gobj,
        Some(
            HSD_SObjLib_803A54EC
                as unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) -> (),
        ),
        priority as u32,
    );
}
pub const NULL: ::core::ffi::c_long = 0 as ::core::ffi::c_long;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32);
    fn GXBegin(a0: u32, a1: u32, a2: u16);
    fn GXClearVtxDesc();
    fn GXGetTexObjFmt(a0: *const _GXTexObj) -> u32;
    fn GXGetTexObjHeight(a0: *const _GXTexObj) -> u16;
    fn GXGetTexObjWidth(a0: *const _GXTexObj) -> u16;
    fn GXInitTexObj(a0: *mut _GXTexObj, a1: *mut c_void, a2: u16, a3: u16, a4: u32, a5: u32, a6: u32, a7: u8);
    fn GXInitTexObjCI(a0: *mut _GXTexObj, a1: *mut c_void, a2: u16, a3: u16, a4: u32, a5: u32, a6: u32, a7: u8, a8: u32);
    fn GXInitTlutObj(a0: *mut _GXTlutObj, a1: *mut c_void, a2: u32, a3: u16);
    fn GXLoadPosMtxImm(a0: *mut c_void, a1: u32);
    fn GXLoadTexObj(a0: *mut _GXTexObj, a1: u32);
    fn GXLoadTlut(a0: *mut _GXTlutObj, a1: u32);
    fn GXPosition2f32(a0: f32, a1: f32);
    fn GXSetAlphaCompare(a0: u32, a1: u8, a2: u32, a3: u32, a4: u8);
    fn GXSetAlphaUpdate(a0: u8);
    fn GXSetBlendMode(a0: u32, a1: u32, a2: u32, a3: u32);
    fn GXSetColorUpdate(a0: u8);
    fn GXSetCullMode(a0: u32);
    fn GXSetCurrentMtx(a0: u32);
    fn GXSetNumTevStages(a0: u8);
    fn GXSetNumTexGens(a0: u8);
    fn GXSetTevAlphaIn(a0: u32, a1: u32, a2: u32, a3: u32, a4: u32);
    fn GXSetTevAlphaOp(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32);
    fn GXSetTevColor(a0: u32, a1: _GXColor);
    fn GXSetTevColorIn(a0: u32, a1: u32, a2: u32, a3: u32, a4: u32);
    fn GXSetTevColorOp(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32);
    fn GXSetTevColorS10(a0: u32, a1: _GXColorS10);
    fn GXSetTevKAlphaSel(a0: u32, a1: u32);
    fn GXSetTevKColor(a0: u32, a1: _GXColor);
    fn GXSetTevKColorSel(a0: u32, a1: u32);
    fn GXSetTevOrder(a0: u32, a1: u32, a2: u32, a3: u32);
    fn GXSetTevSwapMode(a0: u32, a1: u32, a2: u32);
    fn GXSetTevSwapModeTable(a0: u32, a1: u32, a2: u32, a3: u32, a4: u32);
    fn GXSetTexCoordGen2(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32);
    fn GXSetVtxAttrFmt(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8);
    fn GXSetVtxDesc(a0: u32, a1: u32);
    fn GXSetZTexture(a0: u32, a1: u32, a2: u32);
    fn GXTexCoord2f32(a0: f32, a1: f32);
    fn HSD_CObjAlloc() -> *mut HSD_CObj = crate::tu::cobj::HSD_CObjAlloc();
    fn HSD_CObjEndCurrent() = crate::tu::cobj::HSD_CObjEndCurrent();
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj = crate::tu::cobj::HSD_CObjGetCurrent();
    fn HSD_CObjGetViewingMtx(a0: *mut HSD_CObj, a1: *mut c_void) = crate::tu::cobj::HSD_CObjGetViewingMtx(a0 as _, a1 as _);
    fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 = crate::tu::cobj::HSD_CObjSetCurrent(a0) as _;
    fn HSD_CObjSetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjSetFar(a0: *mut HSD_CObj, a1: f32) = crate::tu::cobj::HSD_CObjSetFar(a0, a1);
    fn HSD_CObjSetInterest(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjSetNear(a0: *mut HSD_CObj, a1: f32) = crate::tu::cobj::HSD_CObjSetNear(a0, a1);
    fn HSD_CObjSetOrtho(a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32) = crate::tu::cobj::HSD_CObjSetOrtho(a0, a1, a2, a3, a4);
    fn HSD_CObjSetProjectionType(a0: *mut HSD_CObj, a1: u32);
    fn HSD_CObjSetRoll(a0: *mut HSD_CObj, a1: f32);
    fn HSD_CObjSetScissor(a0: *mut HSD_CObj, a1: *mut _Scissor) = crate::tu::cobj::HSD_CObjSetScissor(a0, a1 as _);
    fn HSD_CObjSetViewport(a0: *mut HSD_CObj, a1: *mut _HSD_RectS16) = crate::tu::cobj::HSD_CObjSetViewport(a0, a1 as _);
    fn HSD_ClearVtxDesc() = crate::tu::pobj::HSD_ClearVtxDesc();
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObjObject_80390ADC(a0: *mut HSD_GObj) -> *mut c_void;
    fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32);
    fn HSD_ObjAlloc(a0: *mut _HSD_ObjAllocData) -> *mut c_void = crate::tu::objalloc::HSD_ObjAlloc(a0 as _);
    fn HSD_ObjAllocInit(a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32) = crate::tu::objalloc::HSD_ObjAllocInit(a0 as _, a1 as _, a2);
    fn HSD_ObjFree(a0: *mut _HSD_ObjAllocData, a1: *mut c_void) = crate::tu::objalloc::HSD_ObjFree(a0 as _, a1);
    fn HSD_SetupChannel(a0: *mut HSD_Chan);
    fn HSD_StateInitTev() = crate::tu::tev::HSD_StateInitTev(crate::tu::globals::render());
    fn HSD_StateInvalidate(a0: i32);
    fn HSD_StateSetNumChans(a0: i32);
    fn HSD_StateSetZCompLoc(a0: i32);
    fn HSD_StateSetZMode(a0: i32, a1: i32, a2: i32);
    fn OSReport(a0: *mut core::ffi::c_char, a1: *mut core::ffi::c_char);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::gobj_native::HSD_GObj_CameraKind;
}
// END GENERATED EXT

const GX_FALSE:u8=0; const GX_TRUE:u8=1;

