//! src/tu/if_2F72.rs vs the C (gen/mtu/tu/if_2F72.c: the whole TU, shipping contraction;
//! regenerate the expected file with `python rust/melee_world/gen/mtu/run_tu.py if_2F72`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::if_2F72 as m;
use mtu_harness::*;
#[path = "mtu/if_2F72_gen.rs"]
mod g;

static mut G: [HSD_GObj; 8] = unsafe { core::mem::zeroed() };
static mut NG: [HSD_GObj; 4] = unsafe { core::mem::zeroed() };
static mut JT: [HSD_JObj; 8] = unsafe { core::mem::zeroed() };
static mut NJ: [HSD_JObj; 4] = unsafe { core::mem::zeroed() };
static mut HI: HudIndex = unsafe { core::mem::zeroed() };
static mut HPOS: [Vec; 6] = [Vec { x: 0.0, y: 0.0, z: 0.0 }; 6];
static mut PROC0: HSD_GObjProc = unsafe { core::mem::zeroed() };
static mut ARCP: *mut HSD_Archive = null_mut();
static mut ARCH: [u8; 16] = [0; 16];
static mut DMD: DynamicModelDesc = unsafe { core::mem::zeroed() };
static mut DMDA: [*mut DynamicModelDesc; 1] = [null_mut(); 1];
static mut JOINTB: [u8; 16] = [0; 16];

pub fn gm_ret() -> i32 {
    let x = fv() % 4;
    if x == 0 {
        -2
    } else if x == 1 {
        -1
    } else {
        x as i32
    }
}

unsafe extern "C" fn cbe1(idx: i32) {
    t(8001);
    t(idx as u32);
}
unsafe extern "C" fn cbe2(idx: i32) {
    t(8002);
    t(idx as u32);
}
unsafe fn pickcb() -> IfStatusCb {
    match rnd() % 3 {
        0 => Some(cbe1),
        1 => Some(cbe2),
        _ => None,
    }
}

unsafe fn setup() {
    G = core::mem::zeroed();
    NG = core::mem::zeroed();
    JT = core::mem::zeroed();
    NJ = core::mem::zeroed();
    HI = core::mem::zeroed();
    PROC0 = core::mem::zeroed();
    DMD = core::mem::zeroed();
    m::ext::ifStatus_803F9628 = core::mem::zeroed();
    for i in 0..8 {
        g::fill_HSD_GObj(&mut G[i]);
        g::fill_HSD_JObj(&mut JT[i]);
        G[i].hsd_obj = &raw mut JT[i] as *mut c_void;
    }
    for i in 0..4 {
        g::fill_HSD_GObj(&mut NG[i]);
        g::fill_HSD_JObj(&mut NJ[i]);
    }
    g::fill_HudIndex(&mut HI);
    for i in 0..6 {
        HPOS[i].x = rw();
        HPOS[i].y = rw();
        HPOS[i].z = rw();
    }
    for i in 0..8usize {
        let e = &mut m::ext::ifStatus_803F9628[i];
        g::fill_Element_803F9628(e);
        e.x0 = if rnd() % 3 == 0 { null_mut() } else { &raw mut G[(rnd() % 8) as usize] };
        e.x10 = (rnd() % 30) as u8;
        e.x11 = (rnd() % 30) as u8;
        e.x18 = pickcb();
        e.x1C = pickcb();
    }
    m::scinfstc_models = core::mem::zeroed();
    DMDA[0] = &raw mut DMD;
    DMD.joint = if rnd() & 1 != 0 { &raw mut JOINTB as *mut HSD_Joint } else { null_mut() };
    m::scinfstc_models.scene_models = if rnd() % 4 == 0 { null_mut() } else { DMDA.as_mut_ptr() };
    for i in 0..6 {
        m::scinfstc_models.slots[i].gobj = if rnd() % 3 == 0 { null_mut() } else { &raw mut G[(rnd() % 8) as usize] };
        m::scinfstc_models.slots[i].gobj2 = if rnd() % 3 == 0 { null_mut() } else { &raw mut G[(rnd() % 8) as usize] };
    }
    ARCP = if rnd() & 1 != 0 { &raw mut ARCH as *mut HSD_Archive } else { null_mut() };
    m::ext::HSD_GObj_JObjKind = (rnd() % 4) as u8;
}

unsafe fn hash_all() {
    for i in 0..8 {
        g::hash_HSD_GObj(&G[i]);
        g::hash_HSD_JObj(&JT[i]);
        g::hash_Element_803F9628(&m::ext::ifStatus_803F9628[i]);
    }
    for i in 0..4 {
        g::hash_HSD_GObj(&NG[i]);
        g::hash_HSD_JObj(&NJ[i]);
    }
    g::hash_HudIndex(&HI);
    for i in 0..6 {
        hm(HPOS[i].x.to_bits());
        hm(HPOS[i].y.to_bits());
        hm(HPOS[i].z.to_bits());
    }
    hm(pi(m::scinfstc_models.scene_models));
    for i in 0..6 {
        hm(pi(m::scinfstc_models.slots[i].gobj));
        hm(pi(m::scinfstc_models.slots[i].gobj2));
    }
    hm(pi(ARCP));
    hm(pi(DMD.joint));
}

#[test]
fn if_2F72_matches_c() {
    unsafe {
        g::install();
        for i in 0..8 {
            preg(&raw const G[i], core::mem::size_of::<HSD_GObj>());
        }
        for i in 0..4 {
            preg(&raw const NG[i], core::mem::size_of::<HSD_GObj>());
        }
        for i in 0..8 {
            preg(&raw const JT[i], core::mem::size_of::<HSD_JObj>());
        }
        for i in 0..4 {
            preg(&raw const NJ[i], core::mem::size_of::<HSD_JObj>());
        }
        preg(&raw const m::ext::ifStatus_803F9628, core::mem::size_of_val(&m::ext::ifStatus_803F9628));
        preg(&raw const HI, core::mem::size_of::<HudIndex>());
        preg(&raw const HPOS, core::mem::size_of_val(&HPOS));
        preg(&raw const PROC0, core::mem::size_of::<HSD_GObjProc>());
        preg(&raw const ARCH, 16);
        preg(&raw const DMD, core::mem::size_of::<DynamicModelDesc>());
        preg(&raw const DMDA, core::mem::size_of_val(&DMDA));
        preg(&raw const JOINTB, 16);
        preg(&raw const ARCP, core::mem::size_of::<*mut HSD_Archive>());
        preg(&raw const m::scinfstc_models, core::mem::size_of::<m::ScInfStcModels>());
        g::freg_all();
        freg(cbe1 as *const () as usize);
        freg(cbe2 as *const () as usize);
        let mut c = Check::new(include_str!("data/mtu_if_2F72.txt"));
        for op in 0..14u32 {
            for k in 0..1500u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => {
                        let gi = (rnd() % 8) as usize;
                        let e = (rnd() % 8) as usize;
                        ret = m::fn_802F7288(&raw mut G[gi], &raw mut m::ext::ifStatus_803F9628[e]) as u32;
                    }
                    1 => m::if_802F73C4(&raw mut G[(rnd() % 8) as usize]),
                    2 => m::if_802F74D0(&raw mut G[(rnd() % 8) as usize]),
                    3 => m::fn_802F75D4(&raw mut G[(rnd() % 8) as usize]),
                    4 => m::fn_802F7670(&raw mut G[(rnd() % 8) as usize]),
                    5 => {
                        let si = (rnd() % 6) as usize;
                        let gi = (rnd() % 8) as usize;
                        let cb = fv() as i32;
                        m::scinfstc_models.slots[si].gobj = &raw mut G[gi];
                        m::fn_802F770C(&raw mut G[gi], cb);
                    }
                    6 => {
                        let gi = (rnd() % 8) as usize;
                        let slot = (rnd() % 6) as u8;
                        let a2 = fv() as u16;
                        let gg = if rnd() & 1 != 0 { &raw mut G[gi] } else { null_mut() };
                        ret = pi(m::fn_802F77F8(gg, slot, a2));
                    }
                    7 => m::fn_802F7994(&raw mut G[(rnd() % 8) as usize]),
                    8 => m::if_802F7AF8((rnd() % 6) as i32),
                    9 => m::if_802F7BB4((rnd() % 6) as i32),
                    10 => m::if_802F7C30((rnd() % 6) as i32),
                    11 => m::if_802F7D08((rnd() % 6) as i32),
                    12 => m::if_802F7E24(),
                    _ => m::if_802F7E7C(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 14 * 1500);
    }
}
