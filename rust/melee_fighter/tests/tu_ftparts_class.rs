//! src/tu/ftparts_render.rs, the fighter-class half (ftParts_JObjInfoInit / IntpJObjInfoInit / PObjInfoInit, ftParts_80073758,
//! ftParts_IntpJObjLoad, ftParts_JObjMakePositionMtx), against the C (gen/cref/tu_ftparts_class_ref.c: the game TU with the
//! real class.c, baselib/mtx.c and mtx_pc.c; expected rows in tests/data/tu_ftparts_class.txt, regenerate with
//! `python rust/melee_fighter/gen/cref/tu_run.py tu_ftparts_class`). The parent classes are synthetic on both sides.
#![allow(static_mut_refs, non_snake_case, unused_unsafe)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_fighter::tu::ftparts_render as fr;
use melee_fighter::types::*;
use melee_runtime::tu::{jobj::hsdJObj, pobj::hsdPObj};

static mut RS: u32 = 0;
static mut H: u32 = 0;
static mut TR: u32 = 0;
fn rnd() -> u32 {
    unsafe {
        RS ^= RS << 13;
        RS ^= RS >> 17;
        RS ^= RS << 5;
        RS
    }
}
fn rg() -> f32 {
    match rnd() % 12 {
        0 => 0.0,
        1 => 1.0,
        2 => -1.0,
        3 => 2.0,
        _ => ((rnd() % 41) as i32 - 20) as f32 / 7.0,
    }
}
fn hm(v: u32) {
    unsafe { H = (H.wrapping_mul(0x01000193)) ^ v }
}
fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
static mut JO: [HSD_JObj; 4] = unsafe { core::mem::zeroed() };
static mut JT: [HSD_Joint; 4] = unsafe { core::mem::zeroed() };

unsafe extern "C" fn cb0() { t(9000) }
unsafe extern "C" fn cb1() { t(9001) }
unsafe extern "C" fn cba(_c: *mut HSD_ClassInfo) -> *mut HSD_Class { t(9002); t(0); null_mut() }
unsafe extern "C" fn cbi(_c: *mut HSD_Class) -> i32 { t(9003); 0 }
unsafe extern "C" fn cbr(_c: *mut HSD_Class) { t(9004) }
unsafe extern "C" fn cbm(_c: *mut HSD_ClassInfo) { t(9005) }
unsafe extern "C" fn cbload(j: *mut HSD_JObj, jt: *mut HSD_Joint, p: *mut HSD_JObj) -> i32 {
    let (mut a, mut b, mut c) = (-1i32, -1i32, -1i32);
    for i in 0..4 {
        if j == &raw mut JO[i] { a = i as i32; }
        if jt == &raw mut JT[i] { b = i as i32; }
        if p == &raw mut JO[i] { c = i as i32; }
    }
    t(9006); t(a as u32); t(b as u32); t(c as u32); t(((*jt).u.dobjdesc.is_null()) as u32);
    rnd() as i32
}
unsafe extern "C" fn cbmk(_j: *mut HSD_JObj) { t(9007) }
unsafe extern "C" fn cbpm(_j: *mut HSD_JObj, m: *mut [f32; 4], r: *mut [f32; 4]) {
    t(9008);
    for i in 0..3 {
        for k in 0..4 {
            (*m.add(i))[k] = rg();
            (*r.add(i))[k] = rg();
        }
    }
}
unsafe extern "C" fn cbd(_j: *mut HSD_JObj, _v: *mut [f32; 4], _p: *mut [f32; 4], _t: u32, _r: u32) { t(9009) }
unsafe extern "C" fn cbrc(_j: *mut HSD_JObj) { t(9010) }
unsafe extern "C" fn cbpd(_p: *mut HSD_PObj, _v: *mut [f32; 4], _m: *mut [f32; 4], _r: u32) { t(9011) }
unsafe extern "C" fn cbps(_p: *mut HSD_PObj, _v: *mut [f32; 4], _m: *mut [f32; 4], _r: u32) { t(9012) }
unsafe extern "C" fn cbpl(_p: *mut HSD_PObj, _d: *mut HSD_PObjDesc) -> i32 { t(9013); 0 }

static mut FN: [usize; 40] = [0; 40];
static mut NFN: usize = 0;
fn freg(f: usize) {
    unsafe {
        FN[NFN] = f;
        NFN += 1;
    }
}
fn fi(f: usize) -> u32 {
    if f == 0 {
        return 0;
    }
    unsafe {
        for i in 0..NFN {
            if FN[i] == f {
                return i as u32 + 1;
            }
        }
    }
    0xFFFF_FFFE
}
static mut CLS: [usize; 5] = [0; 5];
fn ci<T>(c: *const T) -> u32 {
    if c.is_null() {
        return 0;
    }
    unsafe {
        for i in 0..5 {
            if CLS[i] == c as usize {
                return i as u32 + 1;
            }
        }
    }
    0xFFFF_FFFE
}
fn sh(s: *const c_char) -> u32 {
    let mut h = 0x811C_9DC5u32;
    if s.is_null() {
        return 0;
    }
    let mut p = s as *const u8;
    unsafe {
        while *p != 0 {
            h = (h ^ *p as u32).wrapping_mul(0x01000193);
            p = p.add(1);
        }
    }
    h
}
unsafe fn hclass<T>(c: *mut T) {
    let c = c as *mut HSD_ClassInfo;
    let size = core::mem::size_of::<T>();
    let head = core::mem::size_of::<HSD_ClassInfoHead>();
    let w = (c as *const u8).add(head) as *const usize;
    hm(fi((*c).head.info_init.map_or(0, |f| f as usize)));
    hm((*c).head.flags);
    hm(sh((*c).head.library_name));
    hm(sh((*c).head.class_name));
    hm((*c).head.obj_size as i32 as u32);
    hm((*c).head.info_size as i32 as u32);
    hm(ci((*c).head.parent));
    hm(ci((*c).head.next));
    hm(ci((*c).head.child));
    hm((*c).head.nb_exist);
    hm((*c).head.nb_peak);
    for i in 0..(size - head) / core::mem::size_of::<usize>() {
        hm(fi(*w.add(i)));
    }
}
unsafe fn hall() {
    hclass(&raw mut hsdJObj);
    hclass(&raw mut hsdPObj);
    hclass(&raw mut fr::ftJObj);
    hclass(&raw mut fr::ftIntpJObj);
    hclass(&raw mut fr::ftPObj);
}
unsafe fn fillfn<T>(c: *mut T, pool: &[usize; 8]) {
    let size = core::mem::size_of::<T>();
    let head = core::mem::size_of::<HSD_ClassInfoHead>();
    let w = (c as *mut u8).add(head) as *mut usize;
    for i in 0..(size - head) / core::mem::size_of::<usize>() {
        *w.add(i) = pool[(rnd() % 8) as usize];
    }
}

#[test]
fn ftparts_class_matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftparts_class.txt")).unwrap();
    let mut lines = want.lines();
    unsafe {
        CLS = [
            &raw mut hsdJObj as usize,
            &raw mut hsdPObj as usize,
            &raw mut fr::ftJObj as usize,
            &raw mut fr::ftIntpJObj as usize,
            &raw mut fr::ftPObj as usize,
        ];
        let ft_j0 = fr::ftJObj;
        let ft_i0 = fr::ftIntpJObj;
        let ft_p0 = fr::ftPObj;
        for f in [
            cb0 as usize, cb1 as usize, cba as usize, cbi as usize, cbr as usize, cbm as usize, cbload as usize, cbmk as usize,
            cbpm as usize, cbd as usize, cbrc as usize, cbpd as usize, cbps as usize, cbpl as usize,
            fr::ftParts_JObjInfoInit as usize, fr::ftParts_IntpJObjInfoInit as usize, fr::ftParts_PObjInfoInit as usize,
            fr::ftParts_JObjMakePositionMtx as usize, fr::ftParts_IntpJObjLoad as usize, fr::ftParts_PObjSetupMtx as usize,
            melee_runtime::tu::lbrefract::lbRefract_PObjLoad as usize,
        ] {
            freg(f);
        }
        let pool: [usize; 8] = [0, cba as usize, cbi as usize, cbr as usize, cbm as usize, cb0 as usize, cb1 as usize, cbmk as usize];
        let mut objs: Box<[HSD_Class; 4]> = Box::new(core::mem::zeroed());
        let mut ft: Box<Fighter> = Box::new(Fighter::zeroed());
        let mut go: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
        for op in 0..6u32 {
            for k in 0..600u32 {
                let mut ret: u32 = 0;
                RS = 0x2468Au32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729));
                H = 0x811C_9DC5;
                TR = 0;
                fr::ftJObj = ft_j0;
                fr::ftIntpJObj = ft_i0;
                fr::ftPObj = ft_p0;
                hsdJObj = core::mem::zeroed();
                hsdPObj = core::mem::zeroed();
                {
                    let hd = &mut hsdJObj.parent.parent.head;
                    hd.flags = 1;
                    hd.obj_size = core::mem::size_of::<HSD_JObj>() as i16;
                    hd.info_size = core::mem::size_of::<HSD_JObjInfo>() as i16;
                    hd.library_name = c"sysdolphin_base_library".as_ptr() as *mut c_char;
                    hd.class_name = c"hsd_jobj".as_ptr() as *mut c_char;
                    hd.nb_exist = rnd() % 5 + 1;
                    hd.nb_peak = rnd() % 9;
                    hd.child = if rnd() & 1 != 0 { (&raw mut hsdPObj).cast() } else { null_mut() };
                }
                fillfn(&raw mut hsdJObj, &pool);
                hsdJObj.load = Some(cbload);
                hsdJObj.make_pmtx = Some(cbpm);
                {
                    let hd = &mut hsdPObj.parent.head;
                    hd.flags = 1;
                    hd.obj_size = core::mem::size_of::<HSD_PObj>() as i16;
                    hd.info_size = core::mem::size_of::<HSD_PObjInfo>() as i16;
                    hd.library_name = c"sysdolphin_base_library".as_ptr() as *mut c_char;
                    hd.class_name = c"hsd_pobj".as_ptr() as *mut c_char;
                }
                fillfn(&raw mut hsdPObj, &pool);
                hsdPObj.setup_mtx = Some(cbps);
                hsdPObj.load = Some(cbpl);
                if rnd() & 1 != 0 {
                    fr::ftJObj.parent.parent.head.flags = 0;
                }
                match op {
                    0 => fr::ftParts_JObjInfoInit(),
                    1 => fr::ftParts_IntpJObjInfoInit(),
                    2 => fr::ftParts_PObjInfoInit(),
                    3 => {
                        *objs = core::mem::zeroed();
                        for i in 0..4 {
                            objs[i].class_info = (&raw mut hsdJObj).cast();
                        }
                        let j = (rnd() % 4) as usize;
                        fr::ftParts_80073758((&raw mut objs[j]).cast());
                        hm(j as u32);
                        for i in 0..4 {
                            hm(ci(objs[i].class_info));
                        }
                    }
                    4 => {
                        JO = core::mem::zeroed();
                        JT = core::mem::zeroed();
                        let i = (rnd() % 4) as usize;
                        let j = (rnd() % 4) as usize;
                        JT[j].u.dobjdesc = (if rnd() % 3 != 0 { 0x1000usize } else { 0 }) as *mut HSD_DObjDesc;
                        JT[j].flags = rnd();
                        let par = if rnd() & 1 != 0 { &raw mut JO[(i + 1) % 4] } else { null_mut() };
                        ret = fr::ftParts_IntpJObjLoad(&raw mut JO[i], &raw mut JT[j], par) as u32;
                        hm((!JT[j].u.dobjdesc.is_null()) as u32);
                        hm(JT[j].flags);
                    }
                    _ => {
                        *ft = Fighter::zeroed();
                        *go = HSD_GObj::zeroed();
                        go.user_data = (&raw mut *ft).cast::<c_void>();
                        melee_runtime::tu::pobj::ext::HSD_GObj_804D7814 = &raw mut *go;
                        ft.x34_scale.z = if rnd() % 3 == 0 { 1.0 } else { rg() };
                        JO = core::mem::zeroed();
                        JO[0].scale.x = rg();
                        JO[0].scale.y = rg();
                        JO[0].scale.z = rg();
                        if rnd() % 4 == 0 {
                            JO[0].scale.x = 0.0;
                        }
                        let mut m = [[0.0f32; 4]; 3];
                        let mut r = [[0.0f32; 4]; 3];
                        fr::ft_jobj_scale.has_z_scale = rnd() & 1 != 0;
                        for a in 0..3 {
                            for b in 0..4 {
                                fr::ft_jobj_scale.mtx[a][b] = rg();
                            }
                        }
                        fr::ftParts_JObjMakePositionMtx(&raw mut JO[0], m.as_mut_ptr(), r.as_mut_ptr());
                        for a in 0..3 {
                            for b in 0..4 {
                                hm(m[a][b].to_bits());
                                hm(r[a][b].to_bits());
                                hm(fr::ft_jobj_scale.mtx[a][b].to_bits());
                            }
                        }
                        hm(fr::ft_jobj_scale.has_z_scale as u32);
                    }
                }
                hall();
                let got = format!("{} {} {:08X} {:08X} {:08X}", op, k, ret, H, TR);
                assert_eq!(got, lines.next().unwrap(), "op {op} case {k}");
            }
        }
        assert!(lines.next().is_none());
    }
}
