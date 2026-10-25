//! All 28 `src/sysdolphin/baselib/mobj.c` definitions have native C-named entry points
//! in `native` (also re-exported here), checked against the C state/call-trace oracle.
//! The older `HsdRender` descriptor API is kept below for its existing callers.
//!
//! APPROXIMATION (descriptor API only; TODO texp.c): HSD compiles the TExp graph with `HSD_TExpCompile` (texp.c, not
//! ported), which folds constants into TEV registers and merges expressions into as few stages
//! as possible. Here every TExp TEV node becomes one GX stage in order, writing PREV (both colour
//! and alpha; an unset half passes PREV through), constants go to KONST colour registers, and a
//! TObj colour-generator node writes REG0. The arithmetic per pixel is the expression's; the
//! stage layout (and so 8-bit rounding between stages) can differ from the compiled TEV. The
//! specular-lightmap chain is reduced to one `PREV + K(specular) * RAS1` stage, toon and shadow
//! TObjs are not appended.
use super::state::gx::*;
use super::state::{Dat, GxBackend, HsdRender, Rgba};
use super::tobj::{self as tt, TObjRt};

// Native C entry points below are exact ports; the descriptor-renderer API
// remains an approximation until the TExp compiler is connected to the GX backend.
pub use native::*;

pub mod native {
    #![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]
    use super::{
        RENDER_DIFFUSE, RENDER_SHADOW, RENDER_SPECULAR, RENDER_TOON,
        RENDER_VERTEX,
    };
    use core::{
        ffi::c_void,
        mem::size_of,
        ptr::{addr_of_mut, null_mut},
    };
    use melee_types::*;

    pub static mut default_class: *mut HSD_ClassInfo = null_mut();
    pub static mut current_mobj: *mut HSD_MObj = null_mut();
    pub static mut tobj_shadows: *mut HSD_TObj = null_mut();
    pub static mut tobj_toon: *mut HSD_TObj = null_mut();
    pub static mut hsdMObj: HSD_MObjInfo = {
        let mut v: HSD_MObjInfo = unsafe { core::mem::zeroed() };
        v.parent.head.info_init = Some(MObjInfoInit);
        v
    };
    pub static mut tobj_toon_desc: HSD_TObjDesc = {
        let mut v: HSD_TObjDesc = unsafe { core::mem::zeroed() };
        v.id = GX_TEXMAP7;
        v.src = GX_TG_COLOR0;
        v.wrap_s = GX_CLAMP;
        v.wrap_t = GX_CLAMP;
        v.blend_flags = 4;
        v.blending = 1.0;
        v.magFilt = GX_LINEAR;
        v
    };
    unsafe fn method(m: *mut HSD_MObj) -> *mut HSD_MObjInfo {
        (*m).parent.class_info.cast()
    }
    pub unsafe extern "C" fn HSD_MObjSetCurrent(m: *mut HSD_MObj) {
        current_mobj = m;
    }
    pub unsafe extern "C" fn HSD_MObjSetFlags(m: *mut HSD_MObj, flags: u32) {
        if !m.is_null() {
            (*m).rendermode |= flags;
        }
    }
    pub unsafe extern "C" fn HSD_MObjClearFlags(m: *mut HSD_MObj, flags: u32) {
        if !m.is_null() {
            (*m).rendermode &= !flags;
        }
    }
    pub unsafe extern "C" fn HSD_MObjRemoveAnimByFlags(
        m: *mut HSD_MObj,
        flags: u32,
    ) {
        if m.is_null() {
            return;
        }
        if flags & 4 != 0 {
            ext::HSD_AObjRemove((*m).aobj);
            (*m).aobj = null_mut();
        }
        if flags & 16 != 0 {
            ext::HSD_TObjRemoveAnimAll((*m).tobj);
        }
    }
    pub unsafe extern "C" fn HSD_MObjAddAnim(
        m: *mut HSD_MObj,
        a: *mut HSD_MatAnim,
    ) {
        if m.is_null() || a.is_null() {
            return;
        }
        if !(*m).aobj.is_null() {
            ext::HSD_AObjRemove((*m).aobj);
        }
        (*m).aobj = ext::HSD_AObjLoadDesc((*a).aobjdesc);
        ext::HSD_TObjAddAnimAll((*m).tobj, (*a).texanim);
    }
    pub unsafe extern "C" fn HSD_MObjReqAnimByFlags(
        m: *mut HSD_MObj,
        frame: f32,
        flags: u32,
    ) {
        if m.is_null() {
            return;
        }
        if flags & 4 != 0 {
            ext::HSD_AObjReqAnim((*m).aobj, frame);
        }
        ext::HSD_TObjReqAnimAllByFlags((*m).tobj, frame, flags);
    }
    pub unsafe extern "C" fn HSD_MObjReqAnim(m: *mut HSD_MObj, frame: f32) {
        HSD_MObjReqAnimByFlags(m, frame, 0x7ff);
    }
    pub unsafe extern "C" fn MObjUpdateFunc(
        obj: *mut c_void,
        ty: i32,
        v: *mut HSD_ObjData,
    ) {
        let m = obj.cast::<HSD_MObj>();
        if m.is_null() {
            return;
        }
        // No fused expressions in mobj.c. The double multiply precedes unsigned saturation
        // to 32 bits, then truncation, exactly as the existing material-logic port does.
        match ty {
            1..=9 => {
                let x = ((255.0 * (*v).fv as f64) as u32) as u8;
                let mat = (*m).mat;
                match ty {
                    1 => (*mat).ambient.r = x,
                    2 => (*mat).ambient.g = x,
                    3 => (*mat).ambient.b = x,
                    4 => (*mat).diffuse.r = x,
                    5 => (*mat).diffuse.g = x,
                    6 => (*mat).diffuse.b = x,
                    7 => (*mat).specular.r = x,
                    8 => (*mat).specular.g = x,
                    _ => (*mat).specular.b = x,
                }
            }
            10 => (*(*m).mat).alpha = 1.0 - (*v).fv,
            11..=13 if !(*m).pe.is_null() => {
                let x = ((255.0 * (*v).fv as f64) as u32) as u8;
                match ty {
                    11 => (*(*m).pe).ref0 = x,
                    12 => (*(*m).pe).ref1 = x,
                    _ => (*(*m).pe).dst_alpha = x,
                }
            }
            _ => {}
        }
    }
    pub unsafe extern "C" fn HSD_MObjAnim(m: *mut HSD_MObj) {
        if m.is_null() {
            return;
        }
        ext::HSD_AObjInterpretAnim((*m).aobj, m.cast(), Some(MObjUpdateFunc));
        ext::HSD_TObjAnimAll((*m).tobj);
    }
    pub unsafe extern "C" fn MObjLoad(
        m: *mut HSD_MObj,
        d: *mut HSD_MObjDesc,
    ) -> i32 {
        (*m).rendermode = (*d).rendermode;
        (*m).tobj = ext::HSD_TObjLoadDesc((*d).texdesc);
        (*m).mat = HSD_MaterialAlloc();
        core::ptr::copy_nonoverlapping((*d).mat, (*m).mat, 1);
        (*m).rendermode |= RENDER_TOON;
        if !(*d).pedesc.is_null() {
            (*m).pe =
                ext::hsdAllocMemPiece(size_of::<HSD_PEDesc>() as i32).cast();
            core::ptr::copy_nonoverlapping((*d).pedesc, (*m).pe, 1);
        }
        (*m).aobj = null_mut();
        0
    }
    pub unsafe extern "C" fn HSD_MObjLoadDesc(
        d: *mut HSD_MObjDesc,
    ) -> *mut HSD_MObj {
        if d.is_null() {
            return null_mut();
        }
        let info = if (*d).class_name.is_null() {
            null_mut()
        } else {
            ext::hsdSearchClassInfo((*d).class_name)
        };
        let m: *mut HSD_MObj = if info.is_null() {
            HSD_MObjAlloc()
        } else {
            let p = ext::hsdNew(info).cast::<HSD_MObj>();
            assert!(!p.is_null());
            p
        };
        ((*method(m)).load.expect("MObj load"))(m, d);
        HSD_MObjCompileTev(m);
        m
    }
    pub unsafe extern "C" fn HSD_MObjCompileTev(m: *mut HSD_MObj) {
        if m.is_null() {
            return;
        }
        if !(*m).tevdesc.is_null() {
            ext::HSD_TExpFreeTevDesc((*m).tevdesc);
            (*m).tevdesc = null_mut();
        }
        if !(*m).texp.is_null() {
            ext::HSD_TExpFreeList((*m).texp, HSD_TE_ALL, 1);
            (*m).texp = null_mut();
        }
        let mut t = (*m).tobj;
        let mut tail: *mut *mut HSD_TObj = null_mut();
        if (*m).rendermode & RENDER_SHADOW != 0 && !tobj_shadows.is_null() {
            tail = &mut t;
            while !(*tail).is_null() {
                tail = addr_of_mut!((**tail).next);
            }
            *tail = tobj_shadows;
        }
        if (*m).rendermode & RENDER_TOON != 0
            && !tobj_toon.is_null()
            && !(*tobj_toon).imagedesc.is_null()
        {
            (*tobj_toon).next = t;
            t = tobj_toon;
        }
        ext::HSD_TObjAssignResources(t);
        let exp = ((*method(m)).make_texp.expect("MObj make_texp"))(
            m,
            t,
            addr_of_mut!((*m).texp),
        );
        ext::HSD_TExpCompile(
            exp,
            addr_of_mut!((*m).tevdesc),
            addr_of_mut!((*m).texp),
        );
        if !tail.is_null() {
            *tail = null_mut();
        }
    }
    pub unsafe extern "C" fn MObjSetupTev(
        m: *mut HSD_MObj,
        t: *mut HSD_TObj,
        arg: u32,
    ) {
        assert!(!(*m).tevdesc.is_null());
        ext::HSD_TExpSetupTev((*m).tevdesc, (*m).texp);
        ext::HSD_TObjSetupVolatileTev(t, arg);
    }
    pub unsafe extern "C" fn HSD_MObjSetup(
        m: *mut HSD_MObj,
        _rendermode: u32,
    ) {
        ext::HSD_StateInitTev();
        let mode = (*m).rendermode;
        let mat = (*m).mat;
        ext::HSD_SetMaterialColor(
            (*mat).ambient,
            (*mat).diffuse,
            (*mat).specular,
            (*mat).alpha,
        );
        if mode & RENDER_SPECULAR != 0 {
            ext::HSD_SetMaterialShininess((*mat).shininess);
        }
        let mut t = (*m).tobj;
        let mut tail: *mut *mut HSD_TObj = null_mut();
        if mode & RENDER_SHADOW != 0 && !tobj_shadows.is_null() {
            tail = &mut t;
            while !(*tail).is_null() {
                tail = addr_of_mut!((**tail).next);
            }
            *tail = tobj_shadows;
        }
        if mode & RENDER_TOON != 0
            && !tobj_toon.is_null()
            && !(*tobj_toon).imagedesc.is_null()
        {
            (*tobj_toon).next = t;
            t = tobj_toon;
        }
        ext::HSD_TObjSetup(t);
        ext::HSD_TObjSetupTextureCoordGen(t);
        ((*method(m)).setup_tev.expect("MObj setup_tev"))(m, t, mode);
        ext::HSD_SetupRenderModeWithCustomPE(mode, (*m).pe);
        if !tail.is_null() {
            *tail = null_mut();
        }
    }
    pub unsafe extern "C" fn HSD_MObjUnset(
        _m: *mut HSD_MObj,
        _rendermode: u32,
    ) {
        ext::HSD_TObjSetup(null_mut());
    }
    pub unsafe extern "C" fn HSD_MObjSetToonTextureImage(
        image: *mut HSD_ImageDesc,
    ) {
        if tobj_toon.is_null() {
            tobj_toon_desc.imagedesc = image;
            tobj_toon = ext::HSD_TObjLoadDesc(addr_of_mut!(tobj_toon_desc));
            assert!(!tobj_toon.is_null(), "cannot allocate tobj for toon.");
        }
        (*tobj_toon).imagedesc = image;
    }
    pub unsafe extern "C" fn HSD_MObjSetDiffuseColor(
        m: *mut HSD_MObj,
        r: u8,
        g: u8,
        b: u8,
    ) {
        (*(*m).mat).diffuse.r = r;
        (*(*m).mat).diffuse.g = g;
        (*(*m).mat).diffuse.b = b;
    }
    pub unsafe extern "C" fn HSD_MObjSetAlpha(m: *mut HSD_MObj, alpha: f32) {
        (*(*m).mat).alpha = alpha;
    }
    pub unsafe extern "C" fn HSD_MObjGetTObj(
        m: *mut HSD_MObj,
    ) -> *mut HSD_TObj {
        if m.is_null() {
            null_mut()
        } else {
            (*m).tobj
        }
    }
    pub unsafe extern "C" fn HSD_MObjRemove(m: *mut HSD_MObj) {
        if !m.is_null() {
            ((*(*m).parent.class_info).release.expect("MObj release"))(
                m.cast(),
            );
            ((*(*m).parent.class_info).destroy.expect("MObj destroy"))(
                m.cast(),
            );
        }
    }
    pub unsafe extern "C" fn HSD_MObjAlloc() -> *mut HSD_MObj {
        let p = ext::hsdNew(if default_class.is_null() {
            addr_of_mut!(hsdMObj.parent)
        } else {
            default_class
        })
        .cast::<HSD_MObj>();
        assert!(!p.is_null());
        p
    }
    pub unsafe extern "C" fn HSD_MaterialAlloc() -> *mut HSD_Material {
        let p = ext::hsdAllocMemPiece(size_of::<HSD_Material>() as i32)
            .cast::<HSD_Material>();
        assert!(!p.is_null());
        core::ptr::write_bytes(p, 0, 1);
        (*p).alpha = 1.0;
        p
    }
    pub unsafe extern "C" fn HSD_MObjAddShadowTexture(t: *mut HSD_TObj) {
        assert!(!t.is_null());
        let mut cur = tobj_shadows;
        while !cur.is_null() {
            if cur == t {
                return;
            }
            cur = (*cur).next;
        }
        (*t).next = tobj_shadows;
        tobj_shadows = t;
    }
    pub unsafe extern "C" fn HSD_MObjDeleteShadowTexture(t: *mut HSD_TObj) {
        if !t.is_null() {
            let mut cur = addr_of_mut!(tobj_shadows);
            while !(*cur).is_null() {
                if *cur == t {
                    *cur = (*t).next;
                    (*t).next = null_mut();
                    return;
                }
                cur = addr_of_mut!((**cur).next);
            }
        } else {
            while !tobj_shadows.is_null() {
                let next = (*tobj_shadows).next;
                (*tobj_shadows).next = null_mut();
                tobj_shadows = next;
            }
        }
    }
    pub unsafe extern "C" fn MObjRelease(o: *mut HSD_Class) {
        let m = o.cast::<HSD_MObj>();
        ext::HSD_AObjRemove((*m).aobj);
        ext::hsdFreeMemPiece(
            (*m).mat.cast(),
            size_of::<HSD_Material>() as i32,
        );
        ext::HSD_TObjRemoveAll((*m).tobj);
        if !(*m).tevdesc.is_null() {
            ext::HSD_TExpFreeTevDesc((*m).tevdesc);
        }
        if !(*m).texp.is_null() {
            ext::HSD_TExpFreeList((*m).texp, HSD_TE_ALL, 1);
        }
        if !(*m).pe.is_null() {
            ext::hsdFreeMemPiece(
                (*m).pe.cast(),
                size_of::<HSD_PEDesc>() as i32,
            );
        }
        ((*hsdMObj.parent.head.parent)
            .release
            .expect("parent release"))(o);
    }
    pub unsafe extern "C" fn MObjAmnesia(info: *mut HSD_ClassInfo) {
        if info == default_class {
            default_class = null_mut();
        }
        if info == addr_of_mut!(hsdMObj.parent) {
            tobj_toon = null_mut();
            tobj_shadows = null_mut();
        }
        ((*hsdMObj.parent.head.parent)
            .amnesia
            .expect("parent amnesia"))(info);
    }
    pub unsafe extern "C" fn MObjInfoInit() {
        ext::hsdInitClassInfo(
            addr_of_mut!(hsdMObj.parent),
            addr_of_mut!(ext::hsdClass),
            c"sysdolphin_base_library".as_ptr().cast_mut(),
            c"hsd_mobj".as_ptr().cast_mut(),
            size_of::<HSD_MObjInfo>() as i32,
            size_of::<HSD_MObj>() as i32,
        );
        hsdMObj.parent.release = Some(MObjRelease);
        hsdMObj.parent.amnesia = Some(MObjAmnesia);
        hsdMObj.setup = Some(HSD_MObjSetup);
        hsdMObj.unset = Some(HSD_MObjUnset);
        hsdMObj.load = Some(MObjLoad);
        hsdMObj.make_texp = Some(MObjMakeTExp);
        hsdMObj.setup_tev = Some(MObjSetupTev);
    }

    /// Original MObjMakeTExp graph construction. External TExp allocation/compilation remains
    /// a typed integration hook; every node request and input is compared with the C oracle.
    pub unsafe extern "C" fn MObjMakeTExp(
        m: *mut HSD_MObj,
        top: *mut HSD_TObj,
        list: *mut *mut HSD_TExp,
    ) -> *mut HSD_TExp {
        use ext::*;
        const ZERO: *mut HSD_TExp = core::ptr::null_mut();
        const RAS: *mut HSD_TExp = (-2isize) as *mut HSD_TExp;
        const TEX: *mut HSD_TExp = (-1isize) as *mut HSD_TExp;
        assert!(!list.is_null());
        *list = null_mut();
        let mut toon = null_mut();
        let mut t = top;
        while !t.is_null() {
            if (*t).flags & 15 == 4 {
                toon = t;
            }
            t = (*t).next;
        }
        let mut diff;
        let mut alpha;
        let mut done = 0;
        if (*m).rendermode & RENDER_VERTEX != 0 {
            let exp = HSD_TExpTev(list);
            HSD_TExpOrder(exp, null_mut(), GX_COLOR0A0);
            HSD_TExpColorOp(exp, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1);
            HSD_TExpColorIn(
                exp, HSD_TE_0, ZERO, HSD_TE_0, ZERO, HSD_TE_0, ZERO,
                HSD_TE_RGB, RAS,
            );
            HSD_TExpAlphaOp(exp, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1);
            HSD_TExpAlphaIn(
                exp, HSD_TE_0, ZERO, HSD_TE_0, ZERO, HSD_TE_0, ZERO, HSD_TE_A,
                RAS,
            );
            diff = exp;
            alpha = exp;
        } else {
            let dc = HSD_TExpCnst(
                addr_of_mut!((*(*m).mat).diffuse).cast(),
                HSD_TE_RGB,
                HSD_TE_U8,
                list,
            );
            let ac = HSD_TExpCnst(
                addr_of_mut!((*(*m).mat).alpha).cast(),
                HSD_TE_X,
                HSD_TE_F32,
                list,
            );
            let exp = HSD_TExpTev(list);
            HSD_TExpColorOp(exp, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1);
            HSD_TExpColorIn(
                exp, HSD_TE_0, ZERO, HSD_TE_0, ZERO, HSD_TE_0, ZERO,
                HSD_TE_RGB, dc,
            );
            HSD_TExpAlphaOp(exp, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1);
            HSD_TExpAlphaIn(
                exp, HSD_TE_0, ZERO, HSD_TE_0, ZERO, HSD_TE_0, ZERO, HSD_TE_X,
                ac,
            );
            diff = exp;
            alpha = exp;
        }
        t = top;
        while !t.is_null() {
            if (*t).flags & 0x50 != 0 && (*t).id != GX_TEXMAP_NULL {
                let info =
                    (*t).parent.parent.class_info.cast::<HSD_TObjInfo>();
                ((*info).make_texp.expect("TObj make_texp"))(
                    t, 0x50, done, &mut diff, &mut alpha, list,
                );
            }
            t = (*t).next;
        }
        done |= 0x50;
        if (*m).rendermode & RENDER_DIFFUSE != 0 {
            let exp = HSD_TExpTev(list);
            HSD_TExpOrder(exp, toon, GX_COLOR0A0);
            HSD_TExpColorOp(exp, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1);
            HSD_TExpColorIn(
                exp,
                HSD_TE_0,
                ZERO,
                HSD_TE_RGB,
                diff,
                HSD_TE_RGB,
                if toon.is_null() { RAS } else { TEX },
                HSD_TE_0,
                ZERO,
            );
            diff = exp;
            HSD_TExpAlphaOp(exp, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1);
            HSD_TExpAlphaIn(
                exp, HSD_TE_0, ZERO, HSD_TE_A, alpha, HSD_TE_A, RAS, HSD_TE_0,
                ZERO,
            );
            alpha = exp;
        }
        if (*m).rendermode & RENDER_SPECULAR != 0 {
            let cnst = HSD_TExpCnst(
                addr_of_mut!((*(*m).mat).specular).cast(),
                HSD_TE_RGB,
                HSD_TE_U8,
                list,
            );
            let mut exp = HSD_TExpTev(list);
            HSD_TExpColorOp(exp, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1);
            HSD_TExpColorIn(
                exp, HSD_TE_0, ZERO, HSD_TE_0, ZERO, HSD_TE_0, ZERO,
                HSD_TE_RGB, cnst,
            );
            let mut spec = exp;
            t = top;
            while !t.is_null() {
                if (*t).flags & 0x20 != 0 && (*t).id != GX_TEXMAP_NULL {
                    let info =
                        (*t).parent.parent.class_info.cast::<HSD_TObjInfo>();
                    ((*info).make_texp.expect("TObj make_texp"))(
                        t, 0x20, done, &mut spec, &mut alpha, list,
                    );
                }
                t = (*t).next;
            }
            done |= 0x20;
            exp = HSD_TExpTev(list);
            HSD_TExpOrder(exp, null_mut(), GX_COLOR1A1);
            HSD_TExpColorOp(exp, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1);
            HSD_TExpColorIn(
                exp, HSD_TE_0, ZERO, HSD_TE_RGB, spec, HSD_TE_RGB, RAS,
                HSD_TE_0, ZERO,
            );
            spec = exp;
            exp = HSD_TExpTev(list);
            HSD_TExpColorOp(exp, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1);
            HSD_TExpColorIn(
                exp, HSD_TE_RGB, spec, HSD_TE_0, ZERO, HSD_TE_0, ZERO,
                HSD_TE_RGB, diff,
            );
            diff = exp;
        }
        let mut external = diff;
        t = top;
        while !t.is_null() {
            if (*t).flags & 0x80 != 0 && (*t).id != GX_TEXMAP_NULL {
                let info =
                    (*t).parent.parent.class_info.cast::<HSD_TObjInfo>();
                ((*info).make_texp.expect("TObj make_texp"))(
                    t,
                    0x80,
                    done,
                    &mut external,
                    &mut alpha,
                    list,
                );
            }
            t = (*t).next;
        }
        if external != alpha
            || HSD_TExpGetType(external) != HSD_TE_TEV
            || HSD_TExpGetType(alpha) != HSD_TE_TEV
        {
            let exp = HSD_TExpTev(list);
            HSD_TExpColorOp(exp, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1);
            HSD_TExpColorIn(
                exp, HSD_TE_0, ZERO, HSD_TE_0, ZERO, HSD_TE_0, ZERO,
                HSD_TE_RGB, external,
            );
            HSD_TExpAlphaOp(exp, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1);
            HSD_TExpAlphaIn(
                exp, HSD_TE_0, ZERO, HSD_TE_0, ZERO, HSD_TE_0, ZERO, HSD_TE_A,
                alpha,
            );
            return exp;
        }
        external
    }
    // BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
    pub mod ext {
        #![allow(
            non_upper_case_globals,
            non_snake_case,
            unused_imports,
            dead_code
        )]
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
        fn HSD_AObjInterpretAnim(a0: *mut HSD_AObj, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut HSD_ObjData)>);
        fn HSD_AObjLoadDesc(a0: *mut HSD_AObjDesc) -> *mut HSD_AObj = crate::tu::aobj::HSD_AObjLoadDesc(a0);
        fn HSD_AObjRemove(a0: *mut HSD_AObj) = crate::tu::aobj::HSD_AObjRemove(a0);
        fn HSD_AObjReqAnim(a0: *mut HSD_AObj, a1: f32) = crate::tu::aobj::HSD_AObjReqAnim(a0, a1);
        fn HSD_SetMaterialColor(a0: _GXColor, a1: _GXColor, a2: _GXColor, a3: f32);
        fn HSD_SetMaterialShininess(a0: f32) = crate::tu::state::HSD_SetMaterialShininess(crate::tu::globals::render(), a0);
        fn HSD_SetupRenderModeWithCustomPE(a0: u32, a1: *mut HSD_PEDesc);
        fn HSD_StateInitTev() = crate::tu::tev::HSD_StateInitTev(crate::tu::globals::render());
        fn HSD_TExpAlphaIn(a0: *mut HSD_TExp, a1: u32, a2: *mut HSD_TExp, a3: u32, a4: *mut HSD_TExp, a5: u32, a6: *mut HSD_TExp, a7: u32, a8: *mut HSD_TExp);
        fn HSD_TExpAlphaOp(a0: *mut HSD_TExp, a1: u32, a2: u32, a3: u32, a4: u8);
        fn HSD_TExpCnst(a0: *mut c_void, a1: u32, a2: u32, a3: *mut *mut HSD_TExp) -> *mut HSD_TExp;
        fn HSD_TExpColorIn(a0: *mut HSD_TExp, a1: u32, a2: *mut HSD_TExp, a3: u32, a4: *mut HSD_TExp, a5: u32, a6: *mut HSD_TExp, a7: u32, a8: *mut HSD_TExp);
        fn HSD_TExpColorOp(a0: *mut HSD_TExp, a1: u32, a2: u32, a3: u32, a4: u8);
        fn HSD_TExpCompile(a0: *mut HSD_TExp, a1: *mut *mut _HSD_TExpTevDesc, a2: *mut *mut HSD_TExp) -> i32 = crate::tu::texp::HSD_TExpCompile(a0, a1 as _, a2) as _;
        fn HSD_TExpFreeList(a0: *mut HSD_TExp, a1: u32, a2: i32) -> *mut HSD_TExp;
        fn HSD_TExpFreeTevDesc(a0: *mut _HSD_TExpTevDesc) = crate::tu::texp::HSD_TExpFreeTevDesc(a0 as _);
        fn HSD_TExpGetType(a0: *mut HSD_TExp) -> u32;
        fn HSD_TExpOrder(a0: *mut HSD_TExp, a1: *mut HSD_TObj, a2: u32);
        fn HSD_TExpSetupTev(a0: *mut _HSD_TExpTevDesc, a1: *mut HSD_TExp) = crate::tu::texp::HSD_TExpSetupTev(a0 as _, a1);
        fn HSD_TExpTev(a0: *mut *mut HSD_TExp) -> *mut HSD_TExp = crate::tu::texp::HSD_TExpTev(a0);
        fn HSD_TObjAddAnimAll(a0: *mut HSD_TObj, a1: *mut _HSD_TexAnim) = crate::tu::tobj::HSD_TObjAddAnimAll(a0, a1 as _);
        fn HSD_TObjAnimAll(a0: *mut HSD_TObj) = crate::tu::tobj::HSD_TObjAnimAll(a0);
        fn HSD_TObjAssignResources(a0: *mut HSD_TObj) -> i32 = crate::tu::tobj::HSD_TObjAssignResources(a0);
        fn HSD_TObjLoadDesc(a0: *mut _HSD_TObjDesc) -> *mut HSD_TObj = crate::tu::tobj::HSD_TObjLoadDesc(a0 as _);
        fn HSD_TObjRemoveAll(a0: *mut HSD_TObj) = crate::tu::tobj::HSD_TObjRemoveAll(a0);
        fn HSD_TObjRemoveAnimAll(a0: *mut HSD_TObj) = crate::tu::tobj::HSD_TObjRemoveAnimAll(a0);
        fn HSD_TObjReqAnimAllByFlags(a0: *mut HSD_TObj, a1: f32, a2: u32) = crate::tu::tobj::HSD_TObjReqAnimAllByFlags(a0, a1, a2);
        fn HSD_TObjSetup(a0: *mut HSD_TObj);
        fn HSD_TObjSetupTextureCoordGen(a0: *mut HSD_TObj) = crate::tu::tobj::HSD_TObjSetupTextureCoordGen(a0);
        fn HSD_TObjSetupVolatileTev(a0: *mut HSD_TObj, a1: u32) = crate::tu::tobj::HSD_TObjSetupVolatileTev(a0, a1);
        fn hsdAllocMemPiece(a0: i32) -> *mut c_void = crate::tu::class::hsdAllocMemPiece(a0);
        fn hsdFreeMemPiece(a0: *mut c_void, a1: i32) = crate::tu::class::hsdFreeMemPiece(a0, a1);
        fn hsdInitClassInfo(a0: *mut _HSD_ClassInfo, a1: *mut _HSD_ClassInfo, a2: *mut i8, a3: *mut i8, a4: i32, a5: i32) = crate::tu::class::hsdInitClassInfo(a0 as _, a1 as _, a2 as _, a3 as _, a4, a5);
        fn hsdNew(a0: *mut _HSD_ClassInfo) -> *mut c_void = crate::tu::class::hsdNew(a0 as _);
        fn hsdSearchClassInfo(a0: *const i8) -> *mut _HSD_ClassInfo = crate::tu::class::hsdSearchClassInfo(a0 as _) as _;
        }
        /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
        pub mod cb {
            use super::*;
        }
        // Data defined in other TUs (the tests set them).
        pub static mut hsdClass: HSD_ClassInfo =
            unsafe { core::mem::zeroed() };
    }
    // END GENERATED EXT
}

pub const RENDER_VERTEX: u32 = 1 << 1;
pub const RENDER_DIFFUSE: u32 = 1 << 2;
pub const RENDER_SPECULAR: u32 = 1 << 3;
pub const RENDER_TOON: u32 = 1 << 12;
pub const RENDER_SHADOW: u32 = 1 << 26;
pub const RENDER_XLU: u32 = 1 << 30;

const TEX_COLORMAP_MASK: u32 = 0xF << 16;
const TEX_ALPHAMAP_MASK: u32 = 0xF << 20;
const TOBJ_TEVREG_ACTIVE_COLOR_TEV: u32 = 1 << 30;
const TOBJ_TEVREG_ACTIVE_ALPHA_TEV: u32 = 1 << 31;

/// A TEV input (`HSD_TEInput` + its expression, resolved to a GX source).
#[derive(Clone, Copy, Debug, PartialEq)]
enum In {
    Zero,
    One,
    Half,
    PrevC,
    PrevA,
    Reg0C,
    Reg0A,
    RasC,
    RasA,
    TexC,
    TexA,
    /// A constant: RGB triple (HSD_TE_RGB) or one value replicated (HSD_TE_X).
    K([u8; 3]),
}

#[derive(Clone, Copy, Debug)]
struct Half {
    ins: [In; 4],
    op: u32,
    bias: u32,
    scale: u32,
    clamp: bool,
}

const PASS_C: Half = Half { ins: [In::Zero, In::Zero, In::Zero, In::PrevC], op: GX_TEV_ADD, bias: 0, scale: 0, clamp: true };
const PASS_A: Half = Half { ins: [In::Zero, In::Zero, In::Zero, In::PrevA], op: GX_TEV_ADD, bias: 0, scale: 0, clamp: true };

fn add(ins: [In; 4]) -> Half {
    Half { ins, op: GX_TEV_ADD, bias: GX_TB_ZERO, scale: GX_CS_SCALE_1, clamp: true }
}

#[derive(Clone, Copy, Debug)]
struct Stage {
    /// (texcoord, texmap) or none.
    tex: Option<(u32, u32)>,
    chan: u32,
    c: Half,
    a: Half,
    out: u32,
}

/// The stage list `MObjMakeTExp` describes.
struct Tev {
    stages: Vec<Stage>,
}

impl Tev {
    fn push(&mut self, tex: Option<(u32, u32)>, chan: u32, c: Half, a: Half) {
        self.stages.push(Stage { tex, chan, c, a, out: GX_TEVPREV });
    }
}

fn k_of(c: Rgba) -> In {
    In::K([c[0], c[1], c[2]])
}
fn kx(v: u8) -> In {
    In::K([v, v, v])
}
/// `HSD_TE_F32` constants are scaled to 0..255 (texp.c's `HSD_TExpSetReg`: `(u8)(255 * x)`).
fn kf(v: f32) -> In {
    kx((255.0 * v.clamp(0.0, 1.0)) as u8)
}

/// tobj.c `MakeColorGenTExp`: the TObj's own TEV (`HSD_TObjTevDesc`) as one stage into REG0.
/// Returns (colour source, alpha source) for the TObj's main stage.
fn color_gen(d: &Dat, tev: u32, o: &TObjRt, out: &mut Tev) -> (In, In) {
    let active = d.u32_(tev + 0x1C);
    let konst = d.color(tev + 0x10);
    let tev0 = d.color(tev + 0x14);
    let tev1 = d.color(tev + 0x18);
    let cin = |v: u8| -> In {
        match v as u32 {
            15 => In::Zero, // GX_CC_ZERO
            12 => In::One,
            13 => In::Half,
            8 => In::TexC,
            9 => In::TexA,
            0x80 => k_of(konst),
            0x81 => kx(konst[0]),
            0x82 => kx(konst[1]),
            0x83 => kx(konst[2]),
            0x84 => kx(konst[3]),
            0x85 => k_of(tev0),
            0x86 => kx(tev0[3]),
            0x87 => k_of(tev1),
            0x88 => kx(tev1[3]),
            _ => In::Zero, // tobj.c:0x4A7 asserts
        }
    };
    let ain = |v: u8| -> In {
        match v as u32 {
            7 => In::Zero, // GX_CA_ZERO
            4 => In::TexA,
            0x40 => kx(konst[0]),
            0x41 => kx(konst[1]),
            0x42 => kx(konst[2]),
            0x43 => kx(konst[3]),
            0x44 => kx(tev0[3]),
            0x45 => kx(tev1[3]),
            _ => In::Zero, // tobj.c:0x4F0 asserts
        }
    };
    let half = |base: u32, ins: [In; 4]| Half {
        ins,
        op: d.u8_(tev + base) as u32,
        bias: d.u8_(tev + base + 2) as u32,
        scale: d.u8_(tev + base + 4) as u32,
        clamp: d.u8_(tev + base + 6) != 0,
    };
    let mut c = Half { ins: [In::Zero, In::Zero, In::Zero, In::TexC], ..PASS_C };
    let mut a = Half { ins: [In::Zero, In::Zero, In::Zero, In::TexA], ..PASS_A };
    let (mut cs, mut asrc) = (In::TexC, In::TexA);
    if active & TOBJ_TEVREG_ACTIVE_COLOR_TEV != 0 {
        c = half(0, [cin(d.u8_(tev + 8)), cin(d.u8_(tev + 9)), cin(d.u8_(tev + 0xA)), cin(d.u8_(tev + 0xB))]);
        cs = In::Reg0C;
    }
    if active & TOBJ_TEVREG_ACTIVE_ALPHA_TEV != 0 {
        a = half(1, [ain(d.u8_(tev + 0xC)), ain(d.u8_(tev + 0xD)), ain(d.u8_(tev + 0xE)), ain(d.u8_(tev + 0xF))]);
        asrc = In::Reg0A;
    }
    if cs == In::TexC && asrc == In::TexA {
        return (cs, asrc);
    }
    out.stages.push(Stage { tex: Some((o.coord, o.id)), chan: GX_COLOR_NULL, c, a, out: GX_TEVREG0 });
    (if cs == In::Reg0C { In::Reg0C } else { In::TexC }, if asrc == In::Reg0A { In::Reg0A } else { In::TexA })
}

/// tobj.c `TObjMakeTExp`: the TObj's blend into the running colour/alpha (PREV).
fn tobj_make_texp(d: &Dat, o: &TObjRt, lightmap_done: u32, out: &mut Tev) {
    let repeat = lightmap_done & o.lightmap() != 0;
    let (mut c_src, mut a_src) = (In::TexC, In::TexA);
    if let Some(tev) = o.tev {
        if d.u32_(tev + 0x1C) & (TOBJ_TEVREG_ACTIVE_COLOR_TEV | TOBJ_TEVREG_ACTIVE_ALPHA_TEV) != 0 {
            (c_src, a_src) = color_gen(d, tev, o, out);
        }
    }
    // a_sel / a_src as a colour input (HSD_TE_A of the texture or REG0).
    let a_as_c = if a_src == In::Reg0A { In::Reg0A } else { In::TexA };
    let flags = o.flags();
    let c = match flags & TEX_COLORMAP_MASK {
        0x1_0000 => add([In::PrevC, c_src, a_as_c, In::Zero]),           // ALPHA_MASK
        0x2_0000 => add([In::PrevC, c_src, c_src, In::Zero]),            // RGB_MASK
        0x3_0000 => add([In::PrevC, c_src, kf(o.obj.blending), In::Zero]), // BLEND
        0x4_0000 => add([In::Zero, In::PrevC, c_src, In::Zero]),         // MODULATE
        0x5_0000 => add([In::Zero, In::Zero, In::Zero, c_src]),          // REPLACE
        0x7_0000 => add([c_src, In::Zero, In::Zero, In::PrevC]),         // ADD
        0x8_0000 => Half { op: GX_TEV_SUB, ..add([c_src, In::Zero, In::Zero, In::PrevC]) }, // SUB
        _ => PASS_C,                                                     // NONE / PASS
    };
    let a = if repeat {
        PASS_A
    } else {
        match flags & TEX_ALPHAMAP_MASK {
            0x10_0000 => add([In::PrevA, a_src, a_src, In::Zero]),           // ALPHA_MASK
            0x20_0000 => add([In::PrevA, a_src, kf(o.obj.blending), In::Zero]), // BLEND
            0x30_0000 => add([In::Zero, In::PrevA, a_src, In::Zero]),        // MODULATE
            0x40_0000 => add([In::Zero, In::Zero, In::Zero, a_src]),         // REPLACE
            0x60_0000 => add([a_src, In::Zero, In::Zero, In::PrevA]),        // ADD
            0x70_0000 => Half { op: GX_TEV_SUB, ..add([a_src, In::Zero, In::Zero, In::PrevA]) },
            _ => PASS_A,
        }
    };
    out.push(Some((o.coord, o.id)), GX_COLOR_NULL, c, a);
}

/// `MObjMakeTExp` as a stage list (see the module note).
fn make_texp(d: &Dat, rendermode: u32, mat: &MatDesc, tobjs: &[TObjRt]) -> Tev {
    let mut t = Tev { stages: Vec::new() };
    if rendermode & RENDER_VERTEX != 0 {
        t.push(None, GX_COLOR0A0, add([In::Zero, In::Zero, In::Zero, In::RasC]), add([In::Zero, In::Zero, In::Zero, In::RasA]));
    } else {
        t.push(None, GX_COLOR_NULL, add([In::Zero, In::Zero, In::Zero, k_of(mat.diffuse)]), add([In::Zero, In::Zero, In::Zero, kf(mat.alpha)]));
    }
    let mut done = 0u32;
    for o in tobjs {
        if o.lightmap() & (tt::TEX_LIGHTMAP_DIFFUSE | tt::TEX_LIGHTMAP_AMBIENT) != 0 && o.id != GX_TEXMAP_NULL {
            tobj_make_texp(d, o, done, &mut t);
        }
    }
    done |= tt::TEX_LIGHTMAP_DIFFUSE | tt::TEX_LIGHTMAP_AMBIENT;
    if rendermode & RENDER_DIFFUSE != 0 {
        t.push(None, GX_COLOR0A0, add([In::Zero, In::PrevC, In::RasC, In::Zero]), add([In::Zero, In::PrevA, In::RasA, In::Zero]));
    }
    if rendermode & RENDER_SPECULAR != 0 {
        t.push(None, GX_COLOR1A1, add([In::Zero, k_of(mat.specular), In::RasC, In::PrevC]), PASS_A);
        done |= tt::TEX_LIGHTMAP_SPECULAR;
    }
    for o in tobjs {
        if o.lightmap() & tt::TEX_LIGHTMAP_EXT != 0 && o.id != GX_TEXMAP_NULL {
            tobj_make_texp(d, o, done, &mut t);
        }
    }
    t
}

/// An animated MObj (`mobj->mat`, `mobj->pe`, its TObjs' animated fields), as the animation system
/// leaves it; the TObj entries are keyed by the TObj descriptor offset.
#[derive(Clone, Debug)]
pub struct MatOverride {
    pub mat: MatDesc,
    pub pe: Option<[u8; 12]>,
    pub tobjs: Vec<(u32, crate::tobj::HsdTObj<u32>)>,
}

/// The `HSD_Material` + render mode + PE of an `HSD_MObjDesc`.
#[derive(Clone, Copy, Debug)]
pub struct MatDesc {
    pub ambient: Rgba,
    pub diffuse: Rgba,
    pub specular: Rgba,
    pub alpha: f32,
    pub shininess: f32,
}

/// `MObjLoad`: the rendermode (|= RENDER_TOON), material and PE of a descriptor.
pub fn mobj_load(d: &Dat, mobj: u32) -> (u32, MatDesc, Option<[u8; 12]>) {
    let rm = d.u32_(mobj + 4) | RENDER_TOON;
    let mat = match d.ptr(mobj + 0xC) {
        Some(m) => MatDesc { ambient: d.color(m), diffuse: d.color(m + 4), specular: d.color(m + 8), alpha: d.f32_(m + 0xC), shininess: d.f32_(m + 0x10) },
        None => MatDesc { ambient: [0x80, 0x80, 0x80, 0xFF], diffuse: [0xFF; 4], specular: [0xFF; 4], alpha: 1.0, shininess: 50.0 },
    };
    let pe = d.ptr(mobj + 0x14).map(|p| {
        let mut b = [0u8; 12];
        for (i, v) in b.iter_mut().enumerate() {
            *v = d.u8_(p + i as u32);
        }
        b
    });
    (rm, mat, pe)
}

fn cin(i: In) -> (u32, Option<[u8; 3]>) {
    match i {
        In::Zero => (GX_CC_ZERO, None),
        In::One => (GX_CC_ONE, None),
        In::Half => (GX_CC_HALF, None),
        In::PrevC => (GX_CC_CPREV, None),
        In::PrevA => (GX_CC_APREV, None),
        In::Reg0C => (GX_CC_C0, None),
        In::Reg0A => (GX_CC_A0, None),
        In::RasC => (GX_CC_RASC, None),
        In::RasA => (GX_CC_RASA, None),
        In::TexC => (GX_CC_TEXC, None),
        In::TexA => (GX_CC_TEXA, None),
        In::K(k) => (GX_CC_KONST, Some(k)),
    }
}

fn ain(i: In) -> (u32, Option<u8>) {
    match i {
        In::Zero => (GX_CA_ZERO, None),
        In::One => (GX_CA_KONST, Some(255)),
        In::Half => (GX_CA_KONST, Some(128)),
        In::PrevA | In::PrevC => (GX_CA_APREV, None),
        In::Reg0A | In::Reg0C => (GX_CA_A0, None),
        In::RasA | In::RasC => (GX_CA_RASA, None),
        In::TexA | In::TexC => (GX_CA_TEXA, None),
        In::K(k) => (GX_CA_KONST, Some(k[0])),
    }
}

impl HsdRender {
    /// Emit the stage list: TevOrder / ColorIn / AlphaIn / ops, KONST registers per stage.
    fn setup_tev<G: GxBackend>(&mut self, g: &mut G, t: &Tev) {
        let mut kregs: Vec<Rgba> = Vec::new();
        // TEV registers REG1 / REG2: (rgb, alpha) constants a stage needed beyond its one konst.
        let mut xregs: [(Option<[u8; 3]>, Option<u8>); 2] = [(None, None); 2];
        for st in &t.stages {
            if self.current_tev >= 16 {
                break;
            }
            let stage = self.state_assign_tev();
            let (coord, map) = st.tex.unwrap_or((GX_TEXCOORD_NULL, GX_TEXMAP_NULL));
            g.set_tev_order(stage, coord, map, st.chan);
            let mut c: Vec<(u32, Option<[u8; 3]>)> = st.c.ins.iter().map(|&i| cin(i)).collect();
            let mut a: Vec<(u32, Option<u8>)> = st.a.ins.iter().map(|&i| ain(i)).collect();
            let kc = c.iter().find_map(|x| x.1);
            let ka = a.iter().find_map(|x| x.1);
            // A stage selects ONE konst colour and ONE konst alpha. A second distinct constant in
            // the same stage (a TObj TEV of (tev0 - konst) * tex) lives in a TEV register instead
            // (REG1 / REG2, which the stage lists never write): GXSetTevColor, 4u32/C2, 2u32/A2.
            for x in c.iter_mut() {
                if let Some(k) = x.1 {
                    if Some(k) != kc {
                        let r = match xregs.iter().position(|r| r.0 == Some(k)).or_else(|| xregs.iter().position(|r| r.0.is_none())) {
                            Some(r) => r,
                            None => 1,
                        };
                        xregs[r].0 = Some(k);
                        g.set_tev_color_s10(2 + r as u32, [k[0] as i16, k[1] as i16, k[2] as i16, xregs[r].1.unwrap_or(0) as i16]);
                        *x = (if r == 0 { 4u32 } else { 6u32 }, None);
                    }
                }
            }
            for x in a.iter_mut() {
                if let Some(v) = x.1 {
                    if Some(v) != ka {
                        let r = match xregs.iter().position(|r| r.1 == Some(v)).or_else(|| xregs.iter().position(|r| r.1.is_none())) {
                            Some(r) => r,
                            None => 1,
                        };
                        xregs[r].1 = Some(v);
                        let k = xregs[r].0.unwrap_or([0; 3]);
                        g.set_tev_color_s10(2 + r as u32, [k[0] as i16, k[1] as i16, k[2] as i16, v as i16]);
                        *x = (if r == 0 { 2u32 } else { 3u32 }, None);
                    }
                }
            }
            if kc.is_some() || ka.is_some() {
                let want: Rgba = {
                    let k = kc.unwrap_or([0; 3]);
                    [k[0], k[1], k[2], ka.unwrap_or(0)]
                };
                let slot = match kregs.iter().position(|r| (kc.is_none() || r[..3] == want[..3]) && (ka.is_none() || r[3] == want[3])) {
                    Some(s) => s,
                    None if kregs.len() < 4 => {
                        kregs.push(want);
                        g.set_tev_kcolor(kregs.len() as u32 - 1, want);
                        kregs.len() - 1
                    }
                    None => {
                        // Out of KONST registers: reuse the last (approximation).
                        kregs[3] = want;
                        g.set_tev_kcolor(3, want);
                        3
                    }
                };
                g.set_tev_kcolor_sel(stage, GX_TEV_KCSEL_K0 + slot as u32);
                g.set_tev_kalpha_sel(stage, GX_TEV_KASEL_K0_A + slot as u32);
            } else {
                g.set_tev_kcolor_sel(stage, GX_TEV_KCSEL_1);
                g.set_tev_kalpha_sel(stage, GX_TEV_KASEL_1);
            }
            g.set_tev_color_in(stage, c[0].0, c[1].0, c[2].0, c[3].0);
            g.set_tev_alpha_in(stage, a[0].0, a[1].0, a[2].0, a[3].0);
            g.set_tev_color_op(stage, st.c.op, st.c.bias, st.c.scale, st.c.clamp, st.out);
            g.set_tev_alpha_op(stage, st.a.op, st.a.bias, st.a.scale, st.a.clamp, st.out);
            g.set_tev_swap_mode(stage, GX_TEV_SWAP0, GX_TEV_SWAP0);
        }
    }

    /// `HSD_MObjSetup` for the MObj descriptor `mobj`.
    pub fn mobj_setup<G: GxBackend>(&mut self, g: &mut G, d: &Dat, mobj: u32) {
        self.mobj_setup_with(g, d, mobj, None)
    }

    /// `HSD_MObjSetup` for a MObj whose material / TObjs were animated (`ov`).
    pub fn mobj_setup_with<G: GxBackend>(&mut self, g: &mut G, d: &Dat, mobj: u32, ov: Option<&MatOverride>) {
        self.state_init_tev();
        let (rendermode, mut mat, mut pe) = mobj_load(d, mobj);
        if let Some(o) = ov {
            mat = o.mat;
            pe = o.pe;
        }
        self.set_material_color(mat.ambient, mat.diffuse, mat.specular, mat.alpha);
        if rendermode & RENDER_SPECULAR != 0 {
            self.set_material_shininess(mat.shininess);
        }
        let mut tobjs = tt::load_desc_list(d, d.ptr(mobj + 8));
        if let Some(o) = ov {
            for t in tobjs.iter_mut() {
                if let Some((_, a)) = o.tobjs.iter().find(|(desc, _)| *desc == t.desc) {
                    t.obj = a.clone();
                    t.obj.flags |= crate::tobj::TEX_MTX_DIRTY;
                    t.image = a.imagedesc;
                }
            }
        }
        self.tobj_kinds = tobjs.iter().fold(0, |k, o| k | 1 << o.coord_kind());
        self.tobj_setup(g, d, &mut tobjs);
        self.tobj_setup_texture_coord_gen(g, &tobjs);
        // setup_tev (MObjSetupTev): the compiled TEV, then HSD_TObjSetupVolatileTev (shadow /
        // bump only: not ported).
        let tev = make_texp(d, rendermode, &mat, &tobjs);
        self.setup_tev(g, &tev);
        self.setup_render_mode_with_custom_pe(g, rendermode, pe);
    }

    /// `HSD_MObjUnset`: `HSD_TObjSetup(NULL)` (only resets tobj_head in the C).
    pub fn mobj_unset<G: GxBackend>(&mut self, _g: &mut G) {}
}
