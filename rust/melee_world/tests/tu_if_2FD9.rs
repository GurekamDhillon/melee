//! src/tu/if_2FD9.rs vs the C (gen/mtu/tu/if_2FD9.c: the whole TU, shipping contraction;
//! regenerate the expected file with `python rust/melee_world/gen/mtu/run_tu.py if_2FD9`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::if_2FD9 as m;
use mtu_harness::*;
#[path = "mtu/if_2FD9_gen.rs"]
mod g;

static mut G: [HSD_GObj; 4] = unsafe { core::mem::zeroed() };
static mut NG: [HSD_GObj; 4] = unsafe { core::mem::zeroed() };
static mut RJ: HSD_JObj = unsafe { core::mem::zeroed() };
static mut CJ: HSD_JObj = unsafe { core::mem::zeroed() };
static mut NJ: [HSD_JObj; 4] = unsafe { core::mem::zeroed() };
static mut D: [HSD_DObj; 3] = unsafe { core::mem::zeroed() };
static mut M: [HSD_MObj; 3] = unsafe { core::mem::zeroed() };
static mut MA: [HSD_Material; 3] = unsafe { core::mem::zeroed() };
static mut PROC0: HSD_GObjProc = unsafe { core::mem::zeroed() };
static mut ARCP: *mut HSD_Archive = null_mut();
static mut ARCH: [u8; 16] = [0; 16];
static mut DMD: DynamicModelDesc = unsafe { core::mem::zeroed() };
static mut DMDA: [*mut DynamicModelDesc; 1] = [null_mut(); 1];

unsafe fn setup() {
    G = core::mem::zeroed();
    NG = core::mem::zeroed();
    RJ = core::mem::zeroed();
    CJ = core::mem::zeroed();
    NJ = core::mem::zeroed();
    D = core::mem::zeroed();
    M = core::mem::zeroed();
    MA = core::mem::zeroed();
    PROC0 = core::mem::zeroed();
    DMD = core::mem::zeroed();
    for i in 0..4 {
        g::fill_HSD_GObj(&mut G[i]);
        g::fill_HSD_GObj(&mut NG[i]);
        G[i].hsd_obj = &raw mut RJ as *mut c_void;
        G[i].user_data = &m::un_804DDBA8[i] as *const u8 as *mut c_void;
    }
    RJ.flags = rnd() & !(JOBJ_USE_QUATERNION as u32);
    RJ.rotate.x = rw();
    RJ.rotate.y = rw();
    RJ.rotate.z = rw();
    RJ.rotate.w = rw();
    RJ.translate.x = rw();
    RJ.translate.y = rw();
    RJ.translate.z = rw();
    RJ.child = &raw mut CJ;
    CJ.u.dobj = &raw mut D[0];
    for i in 0..3 {
        D[i].next = if i < 2 { &raw mut D[i + 1] } else { null_mut() };
        D[i].mobj = &raw mut M[i];
        M[i].mat = &raw mut MA[i];
        MA[i].diffuse.r = rnd() as u8;
        MA[i].diffuse.g = rnd() as u8;
        MA[i].diffuse.b = rnd() as u8;
        MA[i].diffuse.a = rnd() as u8;
    }
    DMDA[0] = &raw mut DMD;
    DMD.joint = &raw mut ARCH as *mut HSD_Joint;
    ARCP = if rnd() & 1 != 0 { &raw mut ARCH as *mut HSD_Archive } else { null_mut() };
    m::un_804D6D90 = (rnd() & 1) as i32;
    m::un_804A1F10 = core::mem::zeroed();
    m::un_804A1F10.x0 = DMDA.as_mut_ptr();
    for i in 0..4usize {
        m::un_804A1F10.x4[i] = if rnd() % 3 == 0 { null_mut() } else { &raw mut G[(rnd() % 4) as usize] };
        m::un_804A1F10.x14[i].r = rnd() as u8;
        m::un_804A1F10.x14[i].g = rnd() as u8;
        m::un_804A1F10.x14[i].b = rnd() as u8;
        m::un_804A1F10.x14[i].a = rnd() as u8;
        m::un_804A1F10.x24[i] = (rnd() & 1) as u8;
        m::un_804A1F10.x28[i] = if rnd() % 3 == 0 { 0 } else { (rnd() % 4) as u8 };
        m::un_804A1F10.x2C[i] = (rnd() % 4) as u8;
    }
    m::un_804A1F10.x30 = (rnd() & 1) as u8;
    m::un_804A1F10.x31 = if rnd() % 3 == 0 { -1 } else { (rnd() % 4) as i8 };
    m::un_804A1F10.x32 = (rnd() % 51) as u8;
    m::ext::HSD_GObj_JObjKind = (rnd() % 4) as u8;
}

unsafe fn hash_all() {
    for i in 0..4 {
        g::hash_HSD_GObj(&G[i]);
        g::hash_HSD_GObj(&NG[i]);
    }
    hm(RJ.flags);
    hm(RJ.rotate.x.to_bits());
    hm(RJ.rotate.y.to_bits());
    hm(RJ.rotate.z.to_bits());
    hm(RJ.rotate.w.to_bits());
    hm(RJ.translate.x.to_bits());
    hm(RJ.translate.y.to_bits());
    hm(RJ.translate.z.to_bits());
    for i in 0..3 {
        hm(rh(&MA[i].diffuse));
    }
    hm(m::un_804D6D90 as u32);
    hm(pi(m::un_804A1F10.x0));
    for i in 0..4 {
        hm(pi(m::un_804A1F10.x4[i]));
        hm(rh(&m::un_804A1F10.x14[i]));
        hm(m::un_804A1F10.x24[i] as u32);
        hm(m::un_804A1F10.x28[i] as u32);
        hm(m::un_804A1F10.x2C[i] as u32);
    }
    hm(m::un_804A1F10.x30 as u32);
    hm(m::un_804A1F10.x31 as i32 as u32);
    hm(m::un_804A1F10.x32 as u32);
}

#[test]
fn if_2FD9_matches_c() {
    unsafe {
        g::install();
        for i in 0..4 {
            preg(&raw const G[i], core::mem::size_of::<HSD_GObj>());
            preg(&raw const NG[i], core::mem::size_of::<HSD_GObj>());
            preg(&raw const NJ[i], core::mem::size_of::<HSD_JObj>());
        }
        preg(&raw const RJ, core::mem::size_of::<HSD_JObj>());
        preg(&raw const CJ, core::mem::size_of::<HSD_JObj>());
        preg(&raw const D, core::mem::size_of_val(&D));
        preg(&raw const M, core::mem::size_of_val(&M));
        preg(&raw const MA, core::mem::size_of_val(&MA));
        preg(&raw const PROC0, core::mem::size_of::<HSD_GObjProc>());
        preg(&raw const ARCP, core::mem::size_of::<*mut HSD_Archive>());
        preg(&raw const ARCH, 16);
        preg(&raw const DMD, core::mem::size_of::<DynamicModelDesc>());
        preg(&raw const DMDA, core::mem::size_of_val(&DMDA));
        preg(&raw const m::un_804DDBA8, 4);
        preg(&raw const m::un_804A1F10, core::mem::size_of::<m::un_804A1F10_t>());
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_if_2FD9.txt"));
        for op in 0..8u32 {
            for k in 0..2500u32 {
                case(op, k);
                setup();
                let ret = 0u32;
                match op {
                    0 => m::un_802FD910(),
                    1 => m::un_802FD91C(),
                    2 => {
                        let slot = (rnd() % 6) as u8;
                        let a1 = rnd() as u8;
                        let r = rnd() as u8;
                        let gg = rnd() as u8;
                        let b = rnd() as u8;
                        let a = rnd() as u8;
                        let mut col = _GXColor { r, g: gg, b, a };
                        m::un_802FD928(slot, a1, &mut col);
                    }
                    3 => m::un_802FD9D8((rnd() % 6) as u8),
                    4 => {
                        let gi = (rnd() % 4) as usize;
                        let pass = fv() as i32;
                        m::fn_802FDA4C(&raw mut G[gi], pass as isize);
                    }
                    5 => m::fn_802FDA78(&raw mut G[(rnd() % 4) as usize]),
                    6 => m::un_802FE260(),
                    _ => m::un_802FE390(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 8 * 2500);
    }
}
