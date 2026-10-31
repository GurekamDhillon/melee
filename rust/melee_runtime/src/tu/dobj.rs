//! `src/sysdolphin/baselib/dobj.c`, the display half: `DObjLoad`'s transparency bits and
//! `HSD_DObjDisp`. (The flag functions are the verified ports in `crate::dobj`.)
use super::displayfunc::DispModel;
use super::state::{Dat, GxBackend, HsdRender};
use crate::mtx::Mtx;

pub const DOBJ_HIDDEN: u32 = 1;

impl HsdRender {
    /// The DObj's flags after `DObjLoad`: `HSD_DObjModifyFlags(dobj, 2/8/4, 0xE)` from the MObj's
    /// blending bits (0: OPA 2, XLU: TEXEDGE 8, XLU|NO_ZUPDATE: XLU 4). Descriptors carry no DObj
    /// flags of their own, so a loaded DObj starts at 0.
    pub fn dobj_flags(&self, d: &Dat, dobj: u32) -> u32 {
        let Some(mobj) = d.ptr(dobj + 8) else { return 0 };
        match d.u32_(mobj + 4) & 0x6000_0000 {
            0 => 2,
            0x4000_0000 => 8,
            0x6000_0000 => 4,
            // dobj.c:312 "mobj has unexpected blending flags": the C panics.
            _ => 0,
        }
    }

    /// `HSD_DObjDisp` (no LAB script-parts overrides).
    pub fn dobj_disp<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, j: usize, dobj: u32, vmtx: &Mtx, pmtx: &Mtx, rendermode: u32) {
        if self.dobj_flags(&m.dat, dobj) & DOBJ_HIDDEN != 0 {
            return;
        }
        let mobj = m.dat.ptr(dobj + 8);
        if rendermode & 0x400_0000 == 0 {
            if let Some(mo) = mobj {
                let ov = m.mat_anim.and_then(|a| a.get(&dobj));
                self.mobj_setup_with(g, &m.dat, mo, ov);
            }
        }
        let mut p = m.dat.ptr(dobj + 0xC);
        let mut guard = 0;
        while let Some(pobj) = p {
            guard += 1;
            if guard > 1024 {
                break;
            }
            self.pobj_disp(g, m, j, pobj, vmtx, pmtx, rendermode);
            p = m.dat.ptr(pobj + 4);
        }
        if rendermode & 0x400_0000 == 0 {
            self.mobj_unset(g);
        }
    }
}

/// The GameCube-layout half of dobj.c: flag / anim walkers, load / alloc / remove, the class
/// infos and `HSD_DObjDisp`, on `melee_types::HSD_DObj`. The flag accessors also exist as the
/// verified native ports in `crate::dobj`. Callees in unported TUs are TODO stubs in [`ext`]
/// (the PC `ScriptParts_*` hooks default to "no script": 0 / no-op).
pub mod hsd {
    #![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]
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
    fn HSD_AObjRemove(a0: *mut HSD_AObj) = crate::tu::aobj::HSD_AObjRemove(a0 as _);
    fn HSD_MObjAddAnim(a0: *mut HSD_MObj, a1: *mut _HSD_MatAnim) = crate::tu::mobj::HSD_MObjAddAnim(a0 as _, a1 as _);
    fn HSD_MObjAnim(a0: *mut HSD_MObj) = crate::tu::mobj::HSD_MObjAnim(a0 as _);
    fn HSD_MObjLoadDesc(a0: *mut _HSD_MObjDesc) -> *mut HSD_MObj = crate::tu::mobj::HSD_MObjLoadDesc(a0 as _) as _;
    fn HSD_MObjRemove(a0: *mut HSD_MObj) = crate::tu::mobj::HSD_MObjRemove(a0 as _);
    fn HSD_MObjRemoveAnimByFlags(a0: *mut HSD_MObj, a1: u32) = crate::tu::mobj::HSD_MObjRemoveAnimByFlags(a0 as _, a1 as _);
    fn HSD_MObjReqAnimByFlags(a0: *mut HSD_MObj, a1: f32, a2: u32) = crate::tu::mobj::HSD_MObjReqAnimByFlags(a0 as _, a1 as _, a2 as _);
    fn HSD_MObjSetCurrent(a0: *mut HSD_MObj) = crate::tu::mobj::HSD_MObjSetCurrent(a0 as _);
    fn HSD_PObjAddAnimAll(a0: *mut HSD_PObj, a1: *mut HSD_ShapeAnim) = crate::tu::pobj::HSD_PObjAddAnimAll(a0 as _, a1 as _);
    fn HSD_PObjAnimAll(a0: *mut HSD_PObj) = crate::tu::pobj::HSD_PObjAnimAll(a0 as _);
    fn HSD_PObjLoadDesc(a0: *mut HSD_PObjDesc) -> *mut HSD_PObj = crate::tu::pobj::HSD_PObjLoadDesc(a0 as _) as _;
    fn HSD_PObjRemoveAll(a0: *mut HSD_PObj) = crate::tu::pobj::HSD_PObjRemoveAll(a0 as _);
    fn HSD_PObjRemoveAnimAllByFlags(a0: *mut HSD_PObj, a1: u32) = crate::tu::pobj::HSD_PObjRemoveAnimAllByFlags(a0 as _, a1 as _);
    fn HSD_PObjReqAnimAllByFlags(a0: *mut HSD_PObj, a1: f32, a2: u32) = crate::tu::pobj::HSD_PObjReqAnimAllByFlags(a0 as _, a1 as _, a2 as _);
    fn HSD_PObjResolveRefsAll(a0: *mut HSD_PObj, a1: *mut HSD_PObjDesc) = crate::tu::pobj::HSD_PObjResolveRefsAll(a0 as _, a1 as _);
    fn HSD_StateInvalidate(a0: i32);
    fn ScriptParts_Draw(a0: *mut HSD_DObj) -> i32 = 0;
    fn ScriptParts_Forget(a0: *mut HSD_DObj) = ();
    fn ScriptParts_Tint(a0: *mut HSD_DObj) -> i32 = 0;
    fn hsdInitClassInfo(a0: *mut _HSD_ClassInfo, a1: *mut _HSD_ClassInfo, a2: *mut i8, a3: *mut i8, a4: i32, a5: i32) = crate::tu::class::hsdInitClassInfo(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _);
    fn hsdIsDescendantOf(a0: *mut c_void, a1: *mut c_void) -> i32 = crate::tu::class::hsdIsDescendantOf(a0 as _, a1 as _) as _;
    fn hsdNew(a0: *mut _HSD_ClassInfo) -> *mut c_void = crate::tu::class::hsdNew(a0 as _) as _;
    fn hsdSearchClassInfo(a0: *const i8) -> *mut _HSD_ClassInfo = crate::tu::class::hsdSearchClassInfo(a0 as _) as _;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn hsdNew(a0: *mut _HSD_ClassInfo) -> *mut c_void { super::hsdNew(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::class::hsdClass;
    /// Tests (C oracle) run on the `hsdClass` copy above; the engine uses the one `class::hsdClass`.
    pub static mut use_ext_data: bool = false;
    pub unsafe fn hsd_class_ptr() -> *mut HSD_ClassInfo { if use_ext_data { core::ptr::addr_of_mut!(hsdClass) } else { core::ptr::addr_of_mut!(crate::tu::class::hsdClass) } }
}
// END GENERATED EXT

    /// `hsdDObj` (`{ DObjInfoInit }`), `default_class`, `current_dobj`.
    pub static mut hsdDObj: HSD_DObjInfo = {
        let mut info: HSD_DObjInfo = unsafe { core::mem::zeroed() };
        info.parent.head.info_init = Some(DObjInfoInit);
        info
    };
    pub static mut default_class: *mut HSD_ClassInfo = null_mut();
    pub static mut current_dobj: *mut HSD_DObj = null_mut();

    pub unsafe fn init_dobj_statics() {
        hsdDObj.parent.head.info_init = Some(DObjInfoInit);
    }

    pub unsafe fn HSD_DObjSetCurrent(dobj: *mut HSD_DObj) {
        current_dobj = dobj;
    }
    pub unsafe fn HSD_DObjGetFlags(dobj: *mut HSD_DObj) -> u32 {
        if dobj.is_null() { 0 } else { (*dobj).flags }
    }
    pub unsafe fn HSD_DObjSetFlags(dobj: *mut HSD_DObj, flags: u32) {
        if !dobj.is_null() {
            (*dobj).flags |= flags;
        }
    }
    pub unsafe fn HSD_DObjClearFlags(dobj: *mut HSD_DObj, flags: u32) {
        if !dobj.is_null() {
            (*dobj).flags &= !flags;
        }
    }
    pub unsafe fn HSD_DObjModifyFlags(dobj: *mut HSD_DObj, flags: u32, mask: u32) {
        if dobj.is_null() {
            return;
        }
        (*dobj).flags = ((*dobj).flags & !mask) | (flags & mask);
    }

    pub unsafe fn HSD_DObjRemoveAnimByFlags(dobj: *mut HSD_DObj, flags: u32) {
        if dobj.is_null() {
            return;
        }
        if flags & 2 != 0 {
            ext::HSD_AObjRemove((*dobj).aobj);
            (*dobj).aobj = null_mut();
        }
        ext::HSD_PObjRemoveAnimAllByFlags((*dobj).pobj, flags);
        ext::HSD_MObjRemoveAnimByFlags((*dobj).mobj, flags);
    }
    pub unsafe fn HSD_DObjRemoveAnimAllByFlags(dobj: *mut HSD_DObj, flags: u32) {
        let mut dp = dobj;
        while !dp.is_null() {
            HSD_DObjRemoveAnimByFlags(dp, flags);
            dp = (*dp).next;
        }
    }

    pub unsafe fn HSD_DObjAddAnim(dobj: *mut HSD_DObj, mat_anim: *mut HSD_MatAnim, sh_anim: *mut HSD_ShapeAnimDObj) {
        if dobj.is_null() {
            return;
        }
        let shapeanim = if sh_anim.is_null() { null_mut() } else { (*sh_anim).shapeanim };
        ext::HSD_PObjAddAnimAll((*dobj).pobj, shapeanim);
        ext::HSD_MObjAddAnim((*dobj).mobj, mat_anim);
    }
    pub unsafe fn HSD_DObjAddAnimAll(dobj: *mut HSD_DObj, matanim: *mut HSD_MatAnim, shapeanimdobj: *mut HSD_ShapeAnimDObj) {
        if dobj.is_null() {
            return;
        }
        let (mut dp, mut ma, mut sd) = (dobj, matanim, shapeanimdobj);
        while !dp.is_null() {
            HSD_DObjAddAnim(dp, ma, sd);
            dp = (*dp).next;
            ma = if ma.is_null() { null_mut() } else { (*ma).next };
            sd = if sd.is_null() { null_mut() } else { (*sd).next };
        }
    }

    pub unsafe fn HSD_DObjReqAnimByFlags(dobj: *mut HSD_DObj, startframe: f32, flags: u32) {
        if dobj.is_null() {
            return;
        }
        ext::HSD_PObjReqAnimAllByFlags((*dobj).pobj, startframe, flags);
        ext::HSD_MObjReqAnimByFlags((*dobj).mobj, startframe, flags);
    }
    pub unsafe fn HSD_DObjReqAnimAllByFlags(dobj: *mut HSD_DObj, startframe: f32, flags: u32) {
        let mut dp = dobj;
        while !dp.is_null() {
            HSD_DObjReqAnimByFlags(dp, startframe, flags);
            dp = (*dp).next;
        }
    }
    pub unsafe fn HSD_DObjReqAnimAll(dobj: *mut HSD_DObj, startframe: f32) {
        HSD_DObjReqAnimAllByFlags(dobj, startframe, 0x7FF)
    }
    pub unsafe fn HSD_DObjAnim(dobj: *mut HSD_DObj) {
        if dobj.is_null() {
            return;
        }
        ext::HSD_PObjAnimAll((*dobj).pobj);
        ext::HSD_MObjAnim((*dobj).mobj);
    }
    pub unsafe fn HSD_DObjAnimAll(dobj: *mut HSD_DObj) {
        let mut dp = dobj;
        while !dp.is_null() {
            HSD_DObjAnim(dp);
            dp = (*dp).next;
        }
    }

    /// `DObjLoad` (the class `load` method).
    pub unsafe extern "C" fn DObjLoad(dobj: *mut HSD_DObj, desc: *mut HSD_DObjDesc) -> i32 {
        (*dobj).next = HSD_DObjLoadDesc((*desc).next);
        (*dobj).mobj = ext::HSD_MObjLoadDesc((*desc).mobjdesc);
        (*dobj).pobj = ext::HSD_PObjLoadDesc((*desc).pobjdesc);
        if !(*dobj).mobj.is_null() {
            let rm = (*(*dobj).mobj).rendermode;
            match rm & 0x6000_0000 {
                0 => HSD_DObjModifyFlags(dobj, 2, 0xE),
                0x4000_0000 => HSD_DObjModifyFlags(dobj, 8, 0xE),
                0x6000_0000 => HSD_DObjModifyFlags(dobj, 4, 0xE),
                _ => {
                    crate::tu::lbext::OSReport("mobj has unexpected blending flags (0x%x).", &[crate::tu::lbext::OsArg::U(rm)]);
                    panic!("dobj.c:312");
                }
            }
        }
        0
    }

    pub unsafe fn HSD_DObjLoadDesc(desc: *mut HSD_DObjDesc) -> *mut HSD_DObj {
        if desc.is_null() {
            return null_mut();
        }
        let info = if (*desc).class_name.is_null() { null_mut() } else { ext::hsdSearchClassInfo((*desc).class_name) };
        let dobj = if info.is_null() {
            HSD_DObjAlloc()
        } else {
            let d = ext::hsdNew(info) as *mut HSD_DObj;
            assert!(!d.is_null(), "dobj.c:378 dobj");
            d
        };
        let info = (*(dobj as *mut HSD_Class)).class_info as *mut HSD_DObjInfo;
        ((*info).load.expect("DObjInfo.load"))(dobj, desc);
        dobj
    }

    /// class.h `hsdDelete` (static inline): release then destroy through the object's class.
    unsafe fn hsdDelete(object: *mut HSD_Class) {
        if object.is_null() {
            return;
        }
        let info = (*object).class_info;
        ((*info).release.expect("hsdDelete: release"))(object);
        ((*info).destroy.expect("hsdDelete: destroy"))(object);
    }

    pub unsafe fn HSD_DObjRemove(dobj: *mut HSD_DObj) {
        hsdDelete(dobj as *mut HSD_Class);
    }
    pub unsafe fn HSD_DObjRemoveAll(mut dobj: *mut HSD_DObj) {
        while !dobj.is_null() {
            let next = (*dobj).next;
            HSD_DObjRemove(dobj);
            dobj = next;
        }
    }

    pub unsafe fn HSD_DObjSetDefaultClass(info: *mut HSD_ClassInfo) {
        if !info.is_null() && ext::hsdIsDescendantOf(info.cast(), (&raw mut hsdDObj.parent).cast()) == 0 {
            panic!("hsdIsDescendantOf(info, &hsdDObj)");
        }
        default_class = info;
    }
    pub unsafe fn HSD_DObjAlloc() -> *mut HSD_DObj {
        let info = if default_class.is_null() { &mut hsdDObj.parent as *mut HSD_ClassInfo } else { default_class };
        let dobj = ext::hsdNew(info) as *mut HSD_DObj;
        assert!(!dobj.is_null(), "dobj.c:525 dobj");
        dobj
    }

    pub unsafe fn HSD_DObjResolveRefs(dobj: *mut HSD_DObj, desc: *mut HSD_DObjDesc) {
        if dobj.is_null() || desc.is_null() {
            return;
        }
        ext::HSD_PObjResolveRefsAll((*dobj).pobj, (*desc).pobjdesc);
    }
    pub unsafe fn HSD_DObjResolveRefsAll(mut dobj: *mut HSD_DObj, mut desc: *mut HSD_DObjDesc) {
        while !dobj.is_null() && !desc.is_null() {
            HSD_DObjResolveRefs(dobj, desc);
            dobj = (*dobj).next;
            desc = (*desc).next;
        }
    }

    /// `forceStringAllocation`: only asserts (kept for completeness).
    pub unsafe fn forceStringAllocation(dobj: *mut HSD_DObj, mobj: *mut HSD_MObj) {
        assert!(!(*dobj).pobj.is_null(), "dobj.c:700 can not find specified pobj in link.");
        assert!((*dobj).mobj == mobj, "dobj.c:704 dobj->mobj == mobj");
    }

    /// `HSD_DObjDisp` (class `disp` method; TARGET_PC script-parts overrides included).
    pub unsafe extern "C" fn HSD_DObjDisp(dobj: *mut HSD_DObj, vmtx: *mut [f32; 4], pmtx: *mut [f32; 4], rendermode: u32) {
        let mut lab_override = ext::ScriptParts_Draw(dobj);
        if lab_override == 2 {
            return;
        }
        let mobj = (*dobj).mobj;
        ext::HSD_MObjSetCurrent(mobj);
        if lab_override == 0 && rendermode & 0x400_0000 == 0 {
            let mi = (*(mobj as *mut HSD_Class)).class_info as *mut HSD_MObjInfo;
            ((*mi).setup.expect("MObjInfo.setup"))(mobj, rendermode);
            if ext::ScriptParts_Tint(dobj) != 0 {
                lab_override = 1;
            }
        }
        let mut p = (*dobj).pobj;
        while !p.is_null() {
            let pi = (*(p as *mut HSD_Class)).class_info as *mut HSD_PObjInfo;
            let disp: unsafe extern "C" fn(*mut HSD_PObj, *mut [f32; 4], *mut [f32; 4], u32) =
                core::mem::transmute((*pi).disp.expect("PObjInfo.disp"));
            disp(p, vmtx, pmtx, rendermode);
            p = (*p).next;
        }
        if rendermode & 0x400_0000 == 0 {
            let mi = (*(mobj as *mut HSD_Class)).class_info as *mut HSD_MObjInfo;
            ((*mi).unset.expect("MObjInfo.unset"))(mobj, rendermode);
        }
        ext::HSD_MObjSetCurrent(null_mut());
        if lab_override != 0 {
            ext::HSD_StateInvalidate(-1i32 as _);
        }
    }

    /// `DObjRelease` (class `release`).
    pub unsafe extern "C" fn DObjRelease(o: *mut HSD_Class) {
        let dobj = o as *mut HSD_DObj;
        ext::ScriptParts_Forget(dobj);
        ext::HSD_MObjRemove((*dobj).mobj);
        ext::HSD_PObjRemoveAll((*dobj).pobj);
        ext::HSD_AObjRemove((*dobj).aobj);
        let parent = hsdDObj.parent.head.parent;
        if let Some(r) = (*parent).release {
            r(o);
        }
    }

    /// `DObjAmnesia` (class `amnesia`).
    pub unsafe extern "C" fn DObjAmnesia(info: *mut HSD_ClassInfo) {
        if info == default_class {
            default_class = null_mut();
        }
        let parent = hsdDObj.parent.head.parent;
        if let Some(a) = (*parent).amnesia {
            a(info);
        }
    }

    /// `DObjInfoInit`.
    pub unsafe extern "C" fn DObjInfoInit() {
        ext::hsdInitClassInfo(
            &mut hsdDObj.parent,
            ext::hsd_class_ptr(),
            b"sysdolphin_base_library\0".as_ptr().cast_mut().cast(),
            b"hsd_dobj\0".as_ptr().cast_mut().cast(),
            core::mem::size_of::<HSD_DObjInfo>() as i32,
            core::mem::size_of::<HSD_DObj>() as i32,
        );
        hsdDObj.parent.release = Some(DObjRelease);
        hsdDObj.parent.amnesia = Some(DObjAmnesia);
        hsdDObj.disp = Some(core::mem::transmute(HSD_DObjDisp as unsafe extern "C" fn(_, _, _, _)));
        hsdDObj.load = Some(DObjLoad);
    }
}
