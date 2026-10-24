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

    pub mod ext {
        use crate::tu::hookmac::{hook, hook_or};
        use melee_types::*;
        pub static mut HSD_AOBJREMOVE: Option<fn(*mut HSD_AObj)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_AObjRemove(aobj: *mut HSD_AObj) {
        match unsafe { HSD_AOBJREMOVE } { Some(f) => f(aobj), None => { unsafe { let _ = crate::tu::aobj::HSD_AObjRemove(aobj as _); } return; } }
    }
        pub static mut HSD_POBJREMOVEANIMALLBYFLAGS: Option<fn(*mut HSD_PObj, u32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_PObjRemoveAnimAllByFlags(pobj: *mut HSD_PObj, flags: u32) {
        match unsafe { HSD_POBJREMOVEANIMALLBYFLAGS } { Some(f) => f(pobj, flags), None => { unsafe { let _ = crate::tu::pobj::HSD_PObjRemoveAnimAllByFlags(pobj as _, flags as _); } return; } }
    }
        // rp3 HSD_MObjRemoveAnimByFlags
        hook_or!(HSD_MOBJREMOVEANIMBYFLAGS, HSD_MObjRemoveAnimByFlags, (mobj: *mut HSD_MObj, flags: u32), |mobj: *mut HSD_MObj, flags: u32| unsafe { crate::tu::mobj::HSD_MObjRemoveAnimByFlags(mobj, flags); });
        // rp3-end

        pub static mut HSD_POBJADDANIMALL: Option<fn(*mut HSD_PObj, *mut HSD_ShapeAnim)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_PObjAddAnimAll(pobj: *mut HSD_PObj, shapeanim: *mut HSD_ShapeAnim) {
        match unsafe { HSD_POBJADDANIMALL } { Some(f) => f(pobj, shapeanim), None => { unsafe { let _ = crate::tu::pobj::HSD_PObjAddAnimAll(pobj as _, shapeanim as _); } return; } }
    }
        // rp3 HSD_MObjAddAnim
        hook_or!(HSD_MOBJADDANIM, HSD_MObjAddAnim, (mobj: *mut HSD_MObj, mat_anim: *mut HSD_MatAnim), |mobj: *mut HSD_MObj, mat_anim: *mut HSD_MatAnim| unsafe { crate::tu::mobj::HSD_MObjAddAnim(mobj, mat_anim); });
        // rp3-end

        pub static mut HSD_POBJREQANIMALLBYFLAGS: Option<fn(*mut HSD_PObj, f32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_PObjReqAnimAllByFlags(pobj: *mut HSD_PObj, startframe: f32, flags: u32) {
        match unsafe { HSD_POBJREQANIMALLBYFLAGS } { Some(f) => f(pobj, startframe, flags), None => { unsafe { let _ = crate::tu::pobj::HSD_PObjReqAnimAllByFlags(pobj as _, startframe as _, flags as _); } return; } }
    }
        // rp3 HSD_MObjReqAnimByFlags
        hook_or!(HSD_MOBJREQANIMBYFLAGS, HSD_MObjReqAnimByFlags, (mobj: *mut HSD_MObj, startframe: f32, flags: u32), |mobj: *mut HSD_MObj, startframe: f32, flags: u32| unsafe { crate::tu::mobj::HSD_MObjReqAnimByFlags(mobj, startframe, flags); });
        // rp3-end

        pub static mut HSD_POBJANIMALL: Option<fn(*mut HSD_PObj)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_PObjAnimAll(pobj: *mut HSD_PObj) {
        match unsafe { HSD_POBJANIMALL } { Some(f) => f(pobj), None => { unsafe { let _ = crate::tu::pobj::HSD_PObjAnimAll(pobj as _); } return; } }
    }
        // rp3 HSD_MObjAnim
        hook_or!(HSD_MOBJANIM, HSD_MObjAnim, (mobj: *mut HSD_MObj), |mobj: *mut HSD_MObj| unsafe { crate::tu::mobj::HSD_MObjAnim(mobj); });
        // rp3-end

        // rp3 HSD_MObjLoadDesc
        hook_or!(HSD_MOBJLOADDESC, HSD_MObjLoadDesc, (desc: *mut HSD_MObjDesc) -> *mut HSD_MObj, |desc: *mut HSD_MObjDesc| unsafe { crate::tu::mobj::HSD_MObjLoadDesc(desc) });
        // rp3-end

        pub static mut HSD_POBJLOADDESC: Option<fn(*mut HSD_PObjDesc) -> *mut HSD_PObj> = None;
    #[allow(non_snake_case)]
    pub fn HSD_PObjLoadDesc(desc: *mut HSD_PObjDesc) -> *mut HSD_PObj {
        match unsafe { HSD_POBJLOADDESC } { Some(f) => f(desc), None => { return unsafe { crate::tu::pobj::HSD_PObjLoadDesc(desc as _) as _ }; } }
    }
        pub static mut HSD_POBJRESOLVEREFSALL: Option<fn(*mut HSD_PObj, *mut HSD_PObjDesc)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_PObjResolveRefsAll(pobj: *mut HSD_PObj, desc: *mut HSD_PObjDesc) {
        match unsafe { HSD_POBJRESOLVEREFSALL } { Some(f) => f(pobj, desc), None => { unsafe { let _ = crate::tu::pobj::HSD_PObjResolveRefsAll(pobj as _, desc as _); } return; } }
    }
        // rp3 HSD_MObjSetCurrent
        hook_or!(HSD_MOBJSETCURRENT, HSD_MObjSetCurrent, (mobj: *mut HSD_MObj), |mobj: *mut HSD_MObj| unsafe { crate::tu::mobj::HSD_MObjSetCurrent(mobj); });
        // rp3-end

        // rp3 HSD_MObjRemove
        hook_or!(HSD_MOBJREMOVE, HSD_MObjRemove, (mobj: *mut HSD_MObj), |mobj: *mut HSD_MObj| unsafe { crate::tu::mobj::HSD_MObjRemove(mobj); });
        // rp3-end

        pub static mut HSD_POBJREMOVEALL: Option<fn(*mut HSD_PObj)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_PObjRemoveAll(pobj: *mut HSD_PObj) {
        match unsafe { HSD_POBJREMOVEALL } { Some(f) => f(pobj), None => { unsafe { let _ = crate::tu::pobj::HSD_PObjRemoveAll(pobj as _); } return; } }
    }
        pub static mut HSDSEARCHCLASSINFO: Option<fn(*const i8) -> *mut HSD_ClassInfo> = None;
    #[allow(non_snake_case)]
    pub fn hsdSearchClassInfo(name: *const i8) -> *mut HSD_ClassInfo {
        match unsafe { HSDSEARCHCLASSINFO } { Some(f) => f(name), None => { return unsafe { crate::tu::class::hsdSearchClassInfo(name as _) as _ }; } }
    }
        pub static mut HSDNEW: Option<fn(*mut HSD_ClassInfo) -> *mut HSD_Class> = None;
    #[allow(non_snake_case)]
    pub fn hsdNew(info: *mut HSD_ClassInfo) -> *mut HSD_Class {
        match unsafe { HSDNEW } { Some(f) => f(info), None => { return unsafe { crate::tu::class::hsdNew(info as _) as _ }; } }
    }
        hook!(HSDDELETE, hsdDelete, (obj: *mut HSD_Class));
        pub static mut HSDISDESCENDANTOF: Option<fn(*mut HSD_ClassInfo, *mut HSD_ClassInfo) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn hsdIsDescendantOf(info: *mut HSD_ClassInfo, parent: *mut HSD_ClassInfo) -> i32 {
        match unsafe { HSDISDESCENDANTOF } { Some(f) => f(info, parent), None => { return unsafe { crate::tu::class::hsdIsDescendantOf(info as _, parent as _) as _ }; } }
    }
        pub static mut HSDINITCLASSINFO: Option<fn(*mut HSD_ClassInfo, *mut HSD_ClassInfo, *const i8, *const i8, i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn hsdInitClassInfo(info: *mut HSD_ClassInfo, parent: *mut HSD_ClassInfo, lib: *const i8, name: *const i8, info_size: i32, obj_size: i32) {
        match unsafe { HSDINITCLASSINFO } { Some(f) => f(info, parent, lib, name, info_size, obj_size), None => { unsafe { let _ = crate::tu::class::hsdInitClassInfo(info as _, parent as _, lib as _, name as _, info_size as _, obj_size as _); } return; } }
    }
        hook!(HSDCLASS, hsdClass, () -> *mut HSD_ClassInfo);
        hook!(OSREPORT, OSReport, (msg: &str));
        hook!(HSD_STATEINVALIDATE_ALL, HSD_StateInvalidate_All, ());
        hook_or!(SCRIPTPARTS_DRAW, ScriptParts_Draw, (dobj: *mut HSD_DObj) -> i32, |_d| 0);
        hook_or!(SCRIPTPARTS_TINT, ScriptParts_Tint, (dobj: *mut HSD_DObj) -> i32, |_d| 0);
        hook_or!(SCRIPTPARTS_FORGET, ScriptParts_Forget, (dobj: *mut HSD_DObj), |_d| ());
    }

    /// `hsdDObj` (`{ DObjInfoInit }`), `default_class`, `current_dobj`.
    pub static mut hsdDObj: HSD_DObjInfo = unsafe { core::mem::zeroed() };
    static mut default_class: *mut HSD_ClassInfo = null_mut();
    pub static mut current_dobj: *mut HSD_DObj = null_mut();

    pub unsafe fn init_dobj_statics() {
        hsdDObj.parent.head.info_init = Some(DObjInfoInit_c);
    }
    unsafe extern "C" fn DObjInfoInit_c() {
        DObjInfoInit()
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
                    ext::OSReport(&format!("mobj has unexpected blending flags (0x{:x}).", rm));
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

    pub unsafe fn HSD_DObjRemove(dobj: *mut HSD_DObj) {
        ext::hsdDelete(dobj as *mut HSD_Class);
    }
    pub unsafe fn HSD_DObjRemoveAll(mut dobj: *mut HSD_DObj) {
        while !dobj.is_null() {
            let next = (*dobj).next;
            HSD_DObjRemove(dobj);
            dobj = next;
        }
    }

    pub unsafe fn HSD_DObjSetDefaultClass(info: *mut HSD_ClassInfo) {
        if !info.is_null() && ext::hsdIsDescendantOf(info, &mut hsdDObj.parent) == 0 {
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
            ext::HSD_StateInvalidate_All();
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
    pub unsafe fn DObjInfoInit() {
        ext::hsdInitClassInfo(
            &mut hsdDObj.parent,
            ext::hsdClass(),
            b"sysdolphin_base_library\0".as_ptr() as *const i8,
            b"hsd_dobj\0".as_ptr() as *const i8,
            0x44,
            <HSD_DObj as melee_core::gc::GcLayout>::GC_SIZE as i32,
        );
        hsdDObj.parent.release = Some(DObjRelease);
        hsdDObj.parent.amnesia = Some(DObjAmnesia);
        hsdDObj.disp = Some(core::mem::transmute(HSD_DObjDisp as unsafe extern "C" fn(_, _, _, _)));
        hsdDObj.load = Some(DObjLoad);
    }
}
