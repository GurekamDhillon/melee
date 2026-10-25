//! Native-layout port of `src/sysdolphin/baselib/fog.c` (raw pointers over the GameCube-layout
//! `HSD_Fog` / `HSD_FogAdj`): GX state set, alloc / load-desc / init, animation, the class infos.
//! The verified safe logic of the generic fog object is `crate::fog` (oracle `tests/oracle_fog.rs`);
//! this is the C-shaped surface, checked against the C by `tests/tu_fog.rs` (gen/mtu).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]
pub use crate::fog::{HsdFog, HsdFogAdj, HsdFogAdjDesc, HsdFogDesc};
use core::ffi::c_void;
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
    fn GXGetProjectionv(a0: *mut f32);
    fn GXGetViewportv(a0: *mut f32);
    fn GXInitFogAdjTable(a0: *mut _GXFogAdjTable, a1: u16, a2: *mut c_void);
    fn GXSetFog(a0: u32, a1: f32, a2: f32, a3: f32, a4: f32, a5: _GXColor);
    fn GXSetFogRangeAdj(a0: u8, a1: u16, a2: *mut _GXFogAdjTable);
    fn HSD_AObjInterpretAnim(a0: *mut HSD_AObj, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut HSD_ObjData)>) = crate::tu::aobj::HSD_AObjInterpretAnim(a0, a1, core::mem::transmute(a2));
    fn HSD_AObjLoadDesc(a0: *mut HSD_AObjDesc) -> *mut HSD_AObj = crate::tu::aobj::HSD_AObjLoadDesc(a0);
    fn HSD_AObjRemove(a0: *mut HSD_AObj) = crate::tu::aobj::HSD_AObjRemove(a0);
    fn HSD_AObjReqAnim(a0: *mut HSD_AObj, a1: f32) = crate::tu::aobj::HSD_AObjReqAnim(a0, a1);
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj = crate::tu::cobj::HSD_CObjGetCurrent();
    fn HSD_CObjGetFar(a0: *mut HSD_CObj) -> f32 = crate::tu::cobj::HSD_CObjGetFar(a0);
    fn HSD_CObjGetNear(a0: *mut HSD_CObj) -> f32 = crate::tu::cobj::HSD_CObjGetNear(a0);
    fn PSMTXCopy(a0: *mut c_void, a1: *mut c_void) = crate::tu::mtx_pc::raw::PSMTXCopy(a0 as _, a1 as _);
    fn PSMTXIdentity(a0: *mut c_void) = crate::tu::mtx_pc::raw::PSMTXIdentity(a0 as _);
    fn hsdInitClassInfo(a0: *mut _HSD_ClassInfo, a1: *mut _HSD_ClassInfo, a2: *mut i8, a3: *mut i8, a4: i32, a5: i32) = { crate::tu::class::hsdInitClassInfo(a0, a1, a2, a3, a4, a5); };
    fn hsdNew(a0: *mut _HSD_ClassInfo) -> *mut c_void = crate::tu::class::hsdNew(a0);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut hsdObj: HSD_ClassInfo = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// Tests that own their copy of other TUs' data set this; the game uses the owning TU's statics.
pub static mut use_ext_data: bool = false;
/// object.c `hsdObj`.
unsafe fn hsd_obj_ptr() -> *mut HSD_ClassInfo {
    if use_ext_data {
        addr_of_mut!(ext::hsdObj)
    } else {
        addr_of_mut!(crate::tu::object::hsdObj)
    }
}

/// `HSD_Fog_804DE6F0`.
pub static HSD_Fog_804DE6F0: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };

/// `hsdFog` (`{ FogInfoInit }`).
pub static mut hsdFog: HSD_FogInfo = {
    let mut info: HSD_FogInfo = unsafe { core::mem::zeroed() };
    info.parent.parent.head.info_init = Some(FogInfoInit);
    info
};
/// `hsdFogAdj` (`{ FogAdjInfoInit }`).
pub static mut hsdFogAdj: HSD_ClassInfo = {
    let mut info: HSD_ClassInfo = unsafe { core::mem::zeroed() };
    info.head.info_init = Some(FogAdjInfoInit);
    info
};
/// `HSD_FogSet`.
pub unsafe fn HSD_FogSet(fog: *mut HSD_Fog) {
    let mut tbl: GXFogAdjTable = core::mem::zeroed();
    let mut v = [0.0f32; 6];
    // `struct { f32 x0; f32 v[6]; } proj`
    let mut proj = [0.0f32; 7];
    if fog.is_null() {
        ext::GXSetFog(0, 0.0, 0.0, 0.0, 0.0, HSD_Fog_804DE6F0);
        return;
    }
    let cobj = ext::HSD_CObjGetCurrent();
    if cobj.is_null() {
        panic!("fog.c:0x4D You must specify CObj first.");
    }
    ext::GXSetFog((*fog).r#type, (*fog).start, (*fog).end, ext::HSD_CObjGetNear(cobj), ext::HSD_CObjGetFar(cobj), (*fog).color);

    if !(*fog).fog_adj.is_null() {
        ext::GXGetViewportv(v.as_mut_ptr());
        let adj = (*fog).fog_adj;
        let mut range = crate::cabi::f2i32((v[0] + v[2] * ((*adj).center as i32 + 320) as f32 / 640.0) as f64);
        if range < 0 {
            range = 0;
        }
        if range > 640 {
            range = 640;
        }
        if (*adj).width != 0 {
            ext::GXInitFogAdjTable(&mut tbl, (*adj).width, (*adj).mtx.as_mut_ptr().cast());
        } else {
            let mut mtx: Mtx44 = [[0.0; 4]; 4];
            ext::GXGetProjectionv(proj.as_mut_ptr());
            match proj[0] as i32 {
                0 => {
                    mtx[0][0] = proj[1];
                    mtx[0][2] = proj[2];
                    mtx[1][1] = proj[3];
                    mtx[1][2] = proj[4];
                    mtx[2][2] = proj[5];
                    mtx[2][3] = proj[6];
                    mtx[3][2] = -1.0;
                }
                1 => {
                    mtx[0][0] = proj[1];
                    mtx[0][3] = proj[2];
                    mtx[1][1] = proj[3];
                    mtx[1][3] = proj[4];
                    mtx[2][2] = proj[5];
                    mtx[2][3] = proj[6];
                    mtx[3][3] = 1.0;
                }
                _ => {}
            }
            ext::GXInitFogAdjTable(&mut tbl, crate::cabi::f2u16(v[2] as f64), mtx.as_mut_ptr().cast());
        }
        ext::GXSetFogRangeAdj(1, range as u16, &mut tbl);
        return;
    }
    ext::GXSetFogRangeAdj(0, 0, null_mut());
}

/// `HSD_FogAlloc`.
pub unsafe fn HSD_FogAlloc() -> *mut HSD_Fog {
    let fog = ext::hsdNew(addr_of_mut!(hsdFog).cast()) as *mut HSD_Fog;
    assert!(!fog.is_null(), "fog.c:0x8C fog");
    fog
}

/// `HSD_FogLoadDesc`.
pub unsafe fn HSD_FogLoadDesc(desc: *mut HSD_FogDesc) -> *mut HSD_Fog {
    let fog = HSD_FogAlloc();
    assert!(!fog.is_null(), "fog.c:0x99 fog");
    HSD_FogInit(fog, desc);
    if !(*desc).fogadjdesc.is_null() {
        (*fog).fog_adj = HSD_FogAdjLoadDesc((*desc).fogadjdesc);
    }
    fog
}

/// `HSD_FogInit`.
pub unsafe fn HSD_FogInit(fog: *mut HSD_Fog, desc: *mut HSD_FogDesc) {
    if !fog.is_null() {
        if !desc.is_null() {
            (*fog).r#type = (*desc).r#type;
            (*fog).start = (*desc).start;
            (*fog).end = (*desc).end;
            (*fog).color = (*desc).color;
        } else {
            let mut v = [0.0f32; 6];
            ext::GXGetViewportv(v.as_mut_ptr());
            (*fog).r#type = 2; // GX_FOG_LIN
            (*fog).start = v[4];
            (*fog).end = v[5];
            (*fog).color.r = 0xFF;
            (*fog).color.g = 0xFF;
            (*fog).color.b = 0xFF;
            (*fog).color.a = 0xFF;
        }
    }
}

/// `HSD_FogAdjAlloc`.
pub unsafe fn HSD_FogAdjAlloc() -> *mut HSD_FogAdj {
    let adj = ext::hsdNew(addr_of_mut!(hsdFogAdj)) as *mut HSD_FogAdj;
    assert!(!adj.is_null(), "fog.c:0xD6 adj");
    adj
}

/// `HSD_FogAdjLoadDesc`.
pub unsafe fn HSD_FogAdjLoadDesc(desc: *mut HSD_FogAdjDesc) -> *mut HSD_FogAdj {
    let adj = HSD_FogAdjAlloc();
    assert!(!adj.is_null(), "fog.c:0xE6 adj");
    HSD_FogAdjInit(adj, desc);
    adj
}

/// `HSD_FogAdjInit`.
pub unsafe fn HSD_FogAdjInit(adj: *mut HSD_FogAdj, desc: *mut HSD_FogAdjDesc) {
    if !adj.is_null() {
        if !desc.is_null() {
            (*adj).width = (*desc).width;
            (*adj).center = (*desc).center as i16;
            ext::PSMTXCopy((*desc).mtx.as_mut_ptr().cast(), (*adj).mtx.as_mut_ptr().cast());
        } else {
            let mut v = [0.0f32; 6];
            ext::GXGetViewportv(v.as_mut_ptr());
            (*adj).width = crate::cabi::f2u16(v[2] as f64);
            (*adj).center = ((*adj).width as i32 / 2) as i16;
            ext::PSMTXIdentity((*adj).mtx.as_mut_ptr().cast());
            (*adj).mtx[3][0] = 0.0;
            (*adj).mtx[3][1] = 0.0;
            (*adj).mtx[3][2] = 0.0;
            (*adj).mtx[3][3] = 1.0;
        }
    }
}

/// `HSD_Fog_8037DE7C` (HSD_FogAddAnim): replace the fog AObj with one loaded from `desc`.
pub unsafe fn HSD_Fog_8037DE7C(fog: *mut HSD_Fog, desc: *mut HSD_AObjDesc) {
    if !fog.is_null() {
        if !(*fog).aobj.is_null() {
            ext::HSD_AObjRemove((*fog).aobj);
        }
        (*fog).aobj = ext::HSD_AObjLoadDesc(desc);
    }
}

/// `HSD_FogReqAnim`.
pub unsafe fn HSD_FogReqAnim(fog: *mut HSD_Fog, frame: f32) {
    HSD_FogReqAnimByFlags(fog, 0x7FF, frame);
}

/// `HSD_FogReqAnimByFlags`.
pub unsafe fn HSD_FogReqAnimByFlags(fog: *mut HSD_Fog, flags: u32, frame: f32) {
    if fog.is_null() || flags & 0x200 == 0 {
        return;
    }
    ext::HSD_AObjReqAnim((*fog).aobj, frame);
}

/// `HSD_FogInterpretAnim`.
pub unsafe fn HSD_FogInterpretAnim(fog: *mut HSD_Fog) {
    if !fog.is_null() {
        ext::HSD_AObjInterpretAnim((*fog).aobj, fog as *mut c_void, Some(FogUpdateFunc));
    }
}

/// `FogUpdateFunc(obj, type, val)`.
pub unsafe extern "C" fn FogUpdateFunc(obj: *mut c_void, ty: i32, val: *mut HSD_ObjData) {
    let fog = obj as *mut HSD_Fog;
    if !fog.is_null() {
        match ty {
            1 => (*fog).start = (*val).fv,
            2 => (*fog).end = (*val).fv,
            5 => (*fog).color.r = crate::cabi::f2u8((255.0f32 * (*val).fv) as f64),
            6 => (*fog).color.g = crate::cabi::f2u8((255.0f32 * (*val).fv) as f64),
            7 => (*fog).color.b = crate::cabi::f2u8((255.0f32 * (*val).fv) as f64),
            8 => (*fog).color.a = crate::cabi::f2u8((255.0f32 * (*val).fv) as f64),
            20 => {
                if !(*fog).fog_adj.is_null() {
                    (*(*fog).fog_adj).center = crate::cabi::f2i16((*val).fv as f64);
                }
            }
            _ => {}
        }
    }
}

/// `FogRelease` (class `release`): drop the adj (ref_DEC, then its release + destroy), remove the
/// AObj, then the parent class release.
pub unsafe extern "C" fn FogRelease(o: *mut HSD_Class) {
    let fog = o as *mut HSD_Fog;
    if !(*fog).fog_adj.is_null() {
        let adj = (*fog).fog_adj;
        if melee_types::inline::ref_DEC(&mut (*adj).parent) {
            let info = (*(adj as *mut HSD_Class)).class_info;
            if let Some(release) = (*info).release {
                release(adj as *mut HSD_Class);
            }
            if let Some(destroy) = (*info).destroy {
                destroy(adj as *mut HSD_Class);
            }
        }
    }
    ext::HSD_AObjRemove((*fog).aobj);
    let parent = hsdFog.parent.parent.head.parent;
    ((*parent).release.unwrap())(o);
}

/// `FogInfoInit`.
pub unsafe extern "C" fn FogInfoInit() {
    ext::hsdInitClassInfo(
        addr_of_mut!(hsdFog).cast(),
        hsd_obj_ptr().cast(),
        c"sysdolphin_base_library".as_ptr().cast_mut(),
        c"hsd_fog".as_ptr().cast_mut(),
        core::mem::size_of::<HSD_FogInfo>() as i32,
        core::mem::size_of::<HSD_Fog>() as i32,
    );
    hsdFog.parent.parent.release = Some(FogRelease);
}

/// `FogAdjInfoInit`.
pub unsafe extern "C" fn FogAdjInfoInit() {
    ext::hsdInitClassInfo(
        addr_of_mut!(hsdFogAdj).cast(),
        hsd_obj_ptr().cast(),
        c"sysdolphin_base_library".as_ptr().cast_mut(),
        c"hsd_fogadj".as_ptr().cast_mut(),
        core::mem::size_of::<HSD_ClassInfo>() as i32,
        core::mem::size_of::<HSD_FogAdj>() as i32,
    );
}
