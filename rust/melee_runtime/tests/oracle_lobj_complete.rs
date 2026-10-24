//! Whole-state and ordered boundary-effect comparison against the original lobj.c.
#![allow(non_snake_case)]
use melee_runtime::tu::lobj::*;
use melee_types::{
    HSD_AObj, HSD_AObjDesc, HSD_CObj, HSD_LObj, HSD_LObjInfo, HSD_LightAnim,
    HSD_LightAttn, HSD_LightDesc, HSD_LightPointDesc, HSD_LightSpotDesc,
    HSD_ObjData, HSD_WObj, HSD_WObjAnim, HSD_WObjDesc, _GXColor, _GXLightObj,
    _HSD_Class, _HSD_ClassInfo,
};
use std::{cell::RefCell, ptr::null_mut};
thread_local! {
 static OUTPUT:RefCell<Vec<u32>>=const{RefCell::new(Vec::new())};
 static CALLS:RefCell<[u32;32]>=const{RefCell::new([0;32])};
 static LIGHTS:RefCell<Vec<*mut HSD_LObj>>=const{RefCell::new(Vec::new())};
 static WORLDS:RefCell<Vec<*mut HSD_WObj>>=const{RefCell::new(Vec::new())};
 static ANIMS:RefCell<Vec<*mut HSD_AObj>>=const{RefCell::new(Vec::new())};
 static PARENT:RefCell<_HSD_ClassInfo>=RefCell::new(_HSD_ClassInfo::zeroed());
}
fn out(x: u32) {
    OUTPUT.with(|o| o.borrow_mut().push(x));
}
fn bits(x: f32) -> u32 {
    if x.is_nan() {
        0x7fc00000
    } else {
        x.to_bits()
    }
}
fn vout(v: melee_types::Vec) {
    out(bits(v.x));
    out(bits(v.y));
    out(bits(v.z));
}
fn col(c: _GXColor) -> u32 {
    u32::from_be_bytes([c.r, c.g, c.b, c.a])
}
fn ptr(p: usize) -> u32 {
    if p == 0 {
        return 0;
    }
    if let Some(i) =
        LIGHTS.with(|l| l.borrow().iter().position(|&x| x as usize == p))
    {
        return 100 + i as u32;
    }
    if let Some(i) =
        WORLDS.with(|l| l.borrow().iter().position(|&x| x as usize == p))
    {
        return 200 + i as u32;
    }
    if let Some(i) =
        ANIMS.with(|l| l.borrow().iter().position(|&x| x as usize == p))
    {
        return 400 + i as u32;
    }
    1
}
fn event(n: usize, p: usize) {
    CALLS.with(|c| c.borrow_mut()[n] += 1);
    out(0xe0000000 | n as u32);
    out(ptr(p));
}
unsafe extern "C" fn rel(p: *mut _HSD_Class) {
    event(1, p as usize);
}
unsafe extern "C" fn des(p: *mut _HSD_Class) {
    event(2, p as usize);
}
unsafe extern "C" fn amn(p: *mut _HSD_ClassInfo) {
    event(3, p as usize);
}
unsafe fn hook(n: &str, a: &[usize]) -> usize {
    match n {
        "hsdObj" => 0,
        "hsdInitClassInfo" => {
            event(4, a[0]);
            out(a[4] as u32);
            out(a[5] as u32);
            (*(a[0] as *mut _HSD_ClassInfo)).head.parent =
                PARENT.with(|p| p.as_ptr());
            0
        }
        "hsdIsDescendantOf" => {
            event(5, a[0]);
            1
        }
        "hsdNew" => {
            let mut l = Box::new(HSD_LObj::zeroed());
            l.parent.parent.class_info = a[0] as _;
            let l = Box::into_raw(l);
            LIGHTS.with(|v| v.borrow_mut().push(l));
            event(6, l as usize);
            l as usize
        }
        "hsdSearchClassInfo" => {
            event(7, a[0]);
            if *(a[0] as *const u8) == b'x' {
                0
            } else {
                hsdLObj() as usize
            }
        }
        "HSD_AObjRemove" => {
            event(8, a[0]);
            0
        }
        "HSD_AObjLoadDesc" => {
            event(9, a[0]);
            if a[0] == 0 {
                return 0;
            }
            let p = Box::into_raw(Box::new(HSD_AObj::zeroed()));
            ANIMS.with(|v| v.borrow_mut().push(p));
            p as usize
        }
        "HSD_AObjReqAnim" => {
            event(10, a[0]);
            out(a[1] as u32);
            0
        }
        "HSD_AObjInterpretAnim" => {
            event(11, a[0]);
            out(ptr(a[1]));
            if a[0] != 0 {
                let f: unsafe extern "C" fn(
                    *mut core::ffi::c_void,
                    i32,
                    *mut HSD_ObjData,
                ) = core::mem::transmute(a[2]);
                let mut v = HSD_ObjData { fv: 0.25 };
                f(a[1] as _, 9, &mut v);
            }
            0
        }
        "HSD_WObjAlloc" => {
            let mut p = Box::new(HSD_WObj::zeroed());
            p.parent.parent.class_info = PARENT.with(|p| p.as_ptr());
            let p = Box::into_raw(p);
            WORLDS.with(|v| v.borrow_mut().push(p));
            event(12, p as usize);
            p as usize
        }
        "HSD_WObjLoadDesc" => {
            event(13, a[0]);
            if a[0] == 0 {
                return 0;
            }
            let p = hook("HSD_WObjAlloc", &[]) as *mut HSD_WObj;
            (*p).pos = (*(a[0] as *mut HSD_WObjDesc)).pos;
            p as usize
        }
        "HSD_WObjSetPosition" => {
            event(14, a[0]);
            let p = a[0] as *mut HSD_WObj;
            (*p).pos = *(a[1] as *const melee_types::Vec);
            (*p).flags |= 2;
            vout((*p).pos);
            0
        }
        "HSD_WObjGetPosition" => {
            event(15, a[0]);
            *(a[1] as *mut melee_types::Vec) = (*(a[0] as *mut HSD_WObj)).pos;
            0
        }
        "HSD_WObjInterpretAnim" => {
            event(16, a[0]);
            0
        }
        "HSD_WObjReqAnim" => {
            event(17, a[0]);
            out(a[1] as u32);
            0
        }
        "HSD_WObjAddAnim" => {
            event(18, a[0]);
            out((a[1] != 0) as u32);
            0
        }
        "HSD_WObjUnref" => {
            let p = a[0] as *mut HSD_WObj;
            if !p.is_null() {
                let r = (*p).parent.ref_count;
                let release = if r == 65535 {
                    true
                } else {
                    (*p).parent.ref_count = r.wrapping_sub(1);
                    r == 0
                };
                if release {
                    let c = (*p).parent.parent.class_info;
                    (*c).release.unwrap()(p.cast());
                    (*c).destroy.unwrap()(p.cast());
                }
            }
            0
        }
        "GXInitLightColor" => {
            event(19, 0);
            out(a[1] as u32);
            (*(a[0] as *mut _GXLightObj)).dummy[0] = a[1] as u32;
            0
        }
        "GXInitLightPos" | "GXInitLightDir" => {
            let (n, start) = if n == "GXInitLightPos" {
                (20, 1)
            } else {
                (21, 4)
            };
            event(n, 0);
            for i in 0..3 {
                let w = bits(f32::from_bits(a[i + 1] as u32));
                (*(a[0] as *mut _GXLightObj)).dummy[start + i] = w;
                out(w);
            }
            0
        }
        "GXInitLightAttn" => {
            event(22, 0);
            for i in 0..6 {
                let w = bits(f32::from_bits(a[i + 1] as u32));
                (*(a[0] as *mut _GXLightObj)).dummy[7 + i] = w;
                out(w);
            }
            0
        }
        "GXInitLightDistAttn" => {
            event(23, 0);
            for i in 0..3 {
                (*(a[0] as *mut _GXLightObj)).dummy[13 + i] = a[i + 1] as u32;
                out(a[i + 1] as u32);
            }
            0
        }
        "GXInitLightSpot" => {
            event(24, 0);
            for i in 0..2 {
                (*(a[0] as *mut _GXLightObj)).dummy[7 + i] = a[i + 1] as u32;
                out(a[i + 1] as u32);
            }
            0
        }
        "GXLoadLightObjImm" => {
            event(25, 0);
            out(a[1] as u32);
            for w in (*(a[0] as *mut _GXLightObj)).dummy {
                out(w);
            }
            0
        }
        _ => panic!("unexpected hook {n}"),
    }
}
unsafe fn scalar_dump(l: *mut HSD_LObj) {
    out((*l).flags as u32);
    out(col((*l).color));
    let u: [u32; 6] = core::mem::transmute((*l).u);
    for w in u {
        out(w);
    }
}
unsafe fn dump() {
    out(HSD_LObjGetNbActive() as u32);
    for i in 0..9 {
        out(ptr(HSD_LObjGetActiveByID(1 << i) as usize));
    }
    out(HSD_LObjGetLightMaskDiffuse() as u32);
    out(HSD_LObjGetLightMaskSpecular() as u32);
    out(HSD_LObjGetLightMaskAttnFunc() as u32);
    out(HSD_LObjGetLightMaskAlpha() as u32);
    for p in current_lights() {
        out(ptr(p as usize));
    }
    out(0);
    for p in LIGHTS.with(|v| v.borrow().clone()) {
        let l = &*p;
        out(l.flags as u32);
        out(l.priority as u32);
        out(l.parent.ref_count as u32);
        out(ptr(l.next as usize));
        out(col(l.color));
        out(col(l.hw_color));
        out(ptr(l.position as usize));
        out(ptr(l.interest as usize));
        out(ptr(l.aobj as usize));
        let u: [u32; 6] = core::mem::transmute(l.u);
        for w in u {
            out(w);
        }
        out(bits(l.shininess));
        vout(l.lvec);
        out(l.id);
        out(l.spec_id);
        for w in l.lightobj.dummy {
            out(w);
        }
        for w in l.spec_lightobj.dummy {
            out(w);
        }
    }
    for p in WORLDS.with(|v| v.borrow().clone()) {
        out((*p).parent.ref_count as u32);
        out((*p).flags);
        vout((*p).pos);
    }
    for w in CALLS.with(|c| *c.borrow()) {
        out(w);
    }
}
unsafe fn scenario(seed: u32) {
    OUTPUT.with(|x| x.borrow_mut().clear());
    CALLS.with(|x| *x.borrow_mut() = [0; 32]);
    LIGHTS.with(|x| x.borrow_mut().clear());
    WORLDS.with(|x| x.borrow_mut().clear());
    ANIMS.with(|x| x.borrow_mut().clear());
    HSD_LObjClearActive();
    PARENT.with(|p| {
        let mut p = p.borrow_mut();
        p.release = Some(rel);
        p.destroy = Some(des);
        p.amnesia = Some(amn);
    });
    LObjInfoInit();
    (*hsdLObj()).parent.parent.destroy = Some(des);
    out((HSD_LObjGetDefaultClass() == hsdLObj()) as u32);
    let mut custom = *hsdLObj();
    HSD_LObjSetDefaultClass(&mut custom);
    out((HSD_LObjGetDefaultClass() == &mut custom) as u32);
    LObjAmnesia((&mut custom as *mut HSD_LObjInfo).cast());
    out((HSD_LObjGetDefaultClass() == hsdLObj()) as u32);
    HSD_LObjSetDefaultClass(null_mut());
    let mut ds = [HSD_LightDesc::zeroed(); 12];
    let mut wd = [HSD_WObjDesc::zeroed(); 24];
    let mut at = [HSD_LightAttn::zeroed(); 12];
    let mut sp = [HSD_LightSpotDesc::zeroed(); 12];
    let mut pt = [HSD_LightPointDesc::zeroed(); 12];
    let mut la = [HSD_LightAnim::zeroed(); 12];
    let mut ad = HSD_AObjDesc::zeroed();
    let mut wa = HSD_WObjAnim::zeroed();
    for i in 0..12 {
        ds[i].next = if i == 11 {
            null_mut()
        } else {
            ds.as_mut_ptr().add(i + 1)
        };
        ds[i].class_name = match i % 3 {
            0 => c"hsd_lobj".as_ptr(),
            1 => c"x".as_ptr(),
            _ => c"".as_ptr(),
        } as _;
        ds[i].flags = ((i % 4)
            | ((i / 4 + 1) << 2)
            | if seed & 1 != 0 { 16 } else { 0 }
            | if seed & 2 != 0 && i == 1 { 32 } else { 0 })
            as u16;
        ds[i].color = _GXColor {
            r: i as u8 * 17,
            g: (seed * 7) as u8,
            b: 255 - i as u8,
            a: 255,
        };
        ds[i].attnflags = ((seed + i as u32) % 3) as u16;
        ds[i].position = &mut wd[2 * i];
        ds[i].interest = &mut wd[2 * i + 1];
        for k in 0..2 {
            wd[2 * i + k].pos = melee_types::Vec {
                x: (i + 1) as f32,
                y: (seed + 1 + k as u32) as f32,
                z: (3 - k) as f32,
            };
        }
        at[i] = HSD_LightAttn {
            a0: 0.1,
            a1: 0.2,
            a2: 0.3,
            k0: 0.4,
            k1: 0.5,
            k2: 0.6,
        };
        sp[i] = HSD_LightSpotDesc {
            cutoff: 40.,
            spot_func: 2,
            ref_br: 0.75,
            ref_dist: 20.,
            dist_func: 1,
        };
        pt[i] = HSD_LightPointDesc {
            ref_br: 0.625,
            ref_dist: 13.,
            dist_func: 2,
        };
        ds[i].u.p = if ds[i].attnflags != 0 {
            (&mut at[i] as *mut HSD_LightAttn).cast()
        } else if i % 4 == 2 {
            (&mut pt[i] as *mut HSD_LightPointDesc).cast()
        } else {
            (&mut sp[i] as *mut HSD_LightSpotDesc).cast()
        };
        la[i].next = if i == 5 {
            null_mut()
        } else {
            la.as_mut_ptr().add(i + 1)
        };
        la[i].aobjdesc = &mut ad;
        la[i].position_anim = &mut wa;
    }
    let top = HSD_LObjLoadDesc(ds.as_mut_ptr());
    out(HSD_LObjLoadDesc(null_mut()).is_null() as u32);
    let lights = LIGHTS.with(|v| v.borrow().clone());
    for (i, &l) in lights.iter().enumerate() {
        (*l).priority = ((seed + i as u32 * 3) % 7) as u16;
        (*l).parent.ref_count = 5;
    }
    HSD_LObj_803668EC(top);
    dump();
    let mut cam = HSD_CObj::zeroed();
    for i in 0..3 {
        for k in 0..4 {
            cam.view_mtx[i][k] = if i == k {
                1.
            } else {
                (seed as i32 % 5 - 2) as f32 * 0.125
            };
        }
    }
    HSD_LObjSetupInit(&mut cam);
    dump();
    HSD_LObjSetupSpecularInit(&cam.view_mtx);
    dump();
    for &l in &lights {
        let mut c = (*l).color;
        c.r = c.r.wrapping_add(1);
        HSD_LObjSetup(l, c, (seed + 4) as f32);
    }
    dump();
    HSD_LObjSetCurrentAll(lights[9]);
    HSD_LObjSetupInit(&mut cam);
    dump();
    setup_spec_lightobj(lights[9], &cam.view_mtx, 0);
    dump();
    HSD_LObj_803668EC(top);
    HSD_LObjAddAnimAll(top, la.as_mut_ptr());
    HSD_LObjAddAnim(top, la.as_mut_ptr());
    HSD_LObjAddAnim(null_mut(), la.as_mut_ptr());
    HSD_LObjAddAnim(top, null_mut());
    HSD_LObjReqAnimAll(top, seed as f32 + 0.25);
    HSD_LObjReqAnim(null_mut(), 1.);
    HSD_LObjReqAnimAll(null_mut(), 1.);
    HSD_LObjAnimAll(top);
    HSD_LObjAnim(null_mut());
    HSD_LObjAnimAll(null_mut());
    dump();
    let mut val = HSD_ObjData::zeroed();
    for (i, &l) in lights.iter().enumerate() {
        for k in 0..26 {
            val.fv = (seed as i32 - 5) as f32 * 0.125;
            LObjUpdateFunc(l.cast(), k, &mut val);
            scalar_dump(l);
        }
        HSD_LObjSetSpot(l, 33., 2);
        scalar_dump(l);
        HSD_LObjSetDistAttn(l, 12., 0.25, 1);
        scalar_dump(l);
        HSD_LObjSetAttnA(l, 1., 2., 3.);
        scalar_dump(l);
        HSD_LObjSetAttnK(l, 4., 5., 6.);
        scalar_dump(l);
        HSD_LObjSetAttn(l, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6);
        scalar_dump(l);
        HSD_LObjSetFlags(l, 0x8000);
        scalar_dump(l);
        HSD_LObjClearFlags(l, 0x8000);
        scalar_dump(l);
        out(HSD_LObjGetFlags(l));
        out(HSD_LObjGetType(l));
        let mut c = _GXColor {
            r: i as u8,
            g: 2,
            b: 3,
            a: 4,
        };
        HSD_LObjSetColor(l, c);
        HSD_LObjGetColor(l, &mut c);
        out(col(c));
        let mut v = melee_types::Vec {
            x: i as f32,
            y: 2.,
            z: 3.,
        };
        HSD_LObjSetPosition(l, &v);
        v.z = 4.;
        HSD_LObjSetInterest(l, &v);
        v.x = 99.;
        out(HSD_LObjGetPosition(l, &mut v) as u32);
        vout(v);
        out(HSD_LObjGetInterest(l, &mut v) as u32);
        vout(v);
        HSD_LObjGetLightVector(l, &mut v);
        vout(v);
        out(ptr(HSD_LObjGetPositionWObj(l) as usize));
        out(ptr(HSD_LObjGetInterestWObj(l) as usize));
    }
    dump();
    out(LObjLoad(top, &mut ds[0]) as u32);
    HSD_LObjSetPositionWObj(top, (*top).position);
    HSD_LObjSetInterestWObj(top, WORLDS.with(|v| v.borrow()[0]));
    dump();
    HSD_LObjSetFlags(null_mut(), 1);
    HSD_LObjClearFlags(null_mut(), 1);
    out(HSD_LObjGetFlags(null_mut()));
    LObjUpdateFunc(null_mut(), 0, &mut val);
    HSD_LObjSetSpot(null_mut(), 1., 2);
    HSD_LObjSetDistAttn(null_mut(), 1., 2., 3);
    HSD_LObjSetAttn(null_mut(), 1., 2., 3., 4., 5., 6.);
    HSD_LObjSetPositionWObj(null_mut(), null_mut());
    HSD_LObjSetInterestWObj(null_mut(), null_mut());
    let mut v = melee_types::Vec {
        x: 7.,
        y: 8.,
        z: 9.,
    };
    out(HSD_LObjGetPosition(null_mut(), &mut v) as u32);
    out(HSD_LObjGetInterest(null_mut(), &mut v) as u32);
    HSD_LObjGetLightVector(null_mut(), &mut v);
    vout(v);
    out(ptr(HSD_LObjGetPositionWObj(null_mut()) as usize));
    out(ptr(HSD_LObjGetInterestWObj(null_mut()) as usize));
    for i in 0..9 {
        out(HSD_LightID2Index(1 << i));
        out(HSD_Index2LightID(i) as u32);
        out(ptr(HSD_LObjGetActiveByID(1 << i) as usize));
    }
    out(HSD_Index2LightID(9) as u32);
    for i in -1..10 {
        out(ptr(HSD_LObjGetActiveByIndex(i) as usize));
    }
    out(HSD_LObjGetNbActive() as u32);
    out(HSD_LObjGetLightMaskDiffuse() as u32);
    out(HSD_LObjGetLightMaskSpecular() as u32);
    out(HSD_LObjGetLightMaskAttnFunc() as u32);
    out(HSD_LObjGetLightMaskAlpha() as u32);
    for i in 0..4 {
        out(ptr(HSD_LObjGetCurrentByType(i) as usize));
    }
    HSD_LObjAddCurrent(top);
    HSD_LObjAddCurrent(null_mut());
    HSD_LObjDeleteCurrent(null_mut());
    HSD_LObjDeleteCurrent(top);
    HSD_LObjDeleteCurrent(top);
    HSD_LObjDeleteCurrentAll(top);
    dump();
    HSD_LObjSetCurrentAll(top);
    dump();
    LObjReplaceAll(top);
    dump();
    LObjRemoveAll();
    dump();
    HSD_LObjSetCurrentAll(top);
    HSD_LObjDeleteCurrentAll(null_mut());
    HSD_LObjUnrefThis(null_mut());
    (*top).parent.ref_count = 0;
    HSD_LObjUnrefThis(top);
    (*top).parent.ref_count = 65535;
    HSD_LObjUnrefThis(top);
    HSD_LObjRemoveAll(top);
    dump();
    LObjRelease(top.cast());
    LObjAmnesia(hsdLObj().cast());
    dump();
}
unsafe fn list_alloc(
    _: *mut melee_types::HSD_ObjAllocData,
) -> *mut core::ffi::c_void {
    Box::into_raw(Box::new(melee_types::HSD_SList::zeroed())).cast()
}
unsafe fn list_free(
    _: *mut melee_types::HSD_ObjAllocData,
    p: *mut core::ffi::c_void,
) {
    drop(Box::from_raw(p.cast::<melee_types::HSD_SList>()));
}
#[test]
fn complete_native_lobj_matches_c_state_and_effects() {
    unsafe {
        melee_runtime::tu::list::ext::ALLOC = Some(list_alloc);
        melee_runtime::tu::list::ext::FREE = Some(list_free);
    }
    ext::set_hook(Some(hook));
    let mut count = 0;
    for (seed, line) in include_str!("data/oracle_lobj_complete.txt")
        .lines()
        .enumerate()
    {
        unsafe {
            scenario(seed as u32);
        }
        let actual = OUTPUT.with(|o| o.borrow().clone());
        let expected: Vec<u32> = line
            .split_whitespace()
            .map(|w| u32::from_str_radix(w, 16).unwrap())
            .collect();
        for (i, (&a, &e)) in actual.iter().zip(&expected).enumerate() {
            assert_eq!(a,e,"seed {seed}, output word {i}, preceding actual {:?}, expected {:?}",&actual[i.saturating_sub(8)..i],&expected[i.saturating_sub(8)..i]);
        }
        assert_eq!(actual.len(), expected.len(), "seed {seed}");
        count += actual.len();
    }
    ext::set_hook(None);
    assert!(count > 100_000);
}
