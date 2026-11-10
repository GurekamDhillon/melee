//! src/tu/grdisplay.rs vs the C (gen/mtu/tu/grdisplay.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_world/gen/mtu/run_tu.py grdisplay`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals)]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::grdisplay as m;
use mtu_harness::*;
#[path = "mtu/grdisplay_gen.rs"]
mod g;

static mut GOBJ: HSD_GObj = unsafe { core::mem::zeroed() };
static mut X18G: HSD_GObj = unsafe { core::mem::zeroed() };
static mut CURG: HSD_GObj = unsafe { core::mem::zeroed() };
static mut CAMG: HSD_GObj = unsafe { core::mem::zeroed() };
static mut FOGG: HSD_GObj = unsafe { core::mem::zeroed() };
static mut FIG: [HSD_GObj; 3] = unsafe { core::mem::zeroed() };
static mut GP: Ground = unsafe { core::mem::zeroed() };
static mut JT: [HSD_JObj; 7] = unsafe { core::mem::zeroed() };
static mut CAMCO: HSD_CObj = unsafe { core::mem::zeroed() };
static mut OTHCO: HSD_CObj = unsafe { core::mem::zeroed() };
static mut FOGO: [u8; 16] = [0; 16];
static mut SHAD: [LbShadow; 3] = unsafe { core::mem::zeroed() };
static mut PLINK: [*mut HSD_GObj; 16] = [null_mut(); 16];
static mut VM: Mtx = [[0.0; 4]; 3];

unsafe extern "C" fn ocsr(p: *mut Vec3, a: i32, j: *mut HSD_JObj) -> i32 {
    t(5000);
    t((*p).x.to_bits());
    t((*p).y.to_bits());
    t((*p).z.to_bits());
    t(a as u32);
    t(pi(j));
    fv() as i32
}

unsafe fn setup() {
    GOBJ = core::mem::zeroed();
    X18G = core::mem::zeroed();
    CURG = core::mem::zeroed();
    CAMG = core::mem::zeroed();
    FOGG = core::mem::zeroed();
    FIG = core::mem::zeroed();
    GP = core::mem::zeroed();
    JT = core::mem::zeroed();
    SHAD = core::mem::zeroed();
    PLINK = [null_mut(); 16];
    g::fill_HSD_GObj(&mut GOBJ);
    g::fill_HSD_GObj(&mut X18G);
    g::fill_HSD_GObj(&mut CURG);
    g::fill_HSD_GObj(&mut CAMG);
    g::fill_HSD_GObj(&mut FOGG);
    for i in 0..3 {
        g::fill_HSD_GObj(&mut FIG[i]);
    }
    g::fill_Ground(&mut GP);
    for i in 0..3 {
        g::fill_LbShadow(&mut SHAD[i]);
    }
    GOBJ.user_data = &raw mut GP as *mut _;
    GOBJ.hsd_obj = &raw mut JT[0] as *mut _;
    let mut n = rnd() % 4;
    GP.x18 = if n == 0 {
        null_mut()
    } else if n & 1 != 0 {
        &raw mut X18G
    } else {
        &raw mut CURG
    };
    CURG.hsd_obj = &raw mut JT[2] as *mut _;
    X18G.hsd_obj = if rnd() & 1 != 0 { CURG.hsd_obj } else { &raw mut JT[1] as *mut _ };
    CAMG.hsd_obj = if rnd() & 3 == 0 { null_mut() } else { &raw mut CAMCO as *mut _ };
    FOGG.hsd_obj = if rnd() & 1 != 0 { &raw mut FOGO as *mut _ } else { null_mut() };
    m::ext::HSD_GObj_804D7818 = &raw mut CURG;
    m::ext::HSD_GObjPLinkHead = PLINK.as_mut_ptr();
    n = rnd() % 4;
    for i in 0..n as usize {
        FIG[i].next = if i + 1 < n as usize { &raw mut FIG[i + 1] } else { null_mut() };
    }
    PLINK[HSD_GOBJ_PLINK_FIGHTER as usize] = if n != 0 { &raw mut FIG[0] } else { null_mut() };
    m::ext::stage_info.on_check_shadow_render = Some(ocsr);
    for i in 0..7usize {
        g::fill_HSD_JObj(&mut JT[i]);
        let r = rnd();
        if r & 3 == 0 {
            JT[i].flags |= JOBJ_INSTANCE as u32;
        }
        if (r >> 2) & 3 == 0 {
            JT[i].flags |= JOBJ_HIDDEN as u32;
        }
        if (r >> 4) & 1 != 0 {
            JT[i].flags |= JOBJ_MTX_DIRTY as u32;
        }
        if (r >> 5) & 7 == 0 {
            JT[i].flags |= JOBJ_USER_DEF_MTX as u32;
        }
        if (r >> 8) & 1 != 0 {
            JT[i].flags |= (rnd() & 0xF) << 18;
        }
        if (r >> 9) & 1 != 0 {
            JT[i].flags |= (rnd() & 0xF) << 28;
        }
        JT[i].child = if i < 6 && rnd() & 3 != 0 { &raw mut JT[i + 1] } else { null_mut() };
        JT[i].next = if i < 6 && rnd() % 3 != 0 {
            let j = i + 1 + (rnd() % (6 - i) as u32) as usize;
            &raw mut JT[j]
        } else {
            null_mut()
        };
        if JT[i].child.is_null() {
            JT[i].flags &= !(JOBJ_INSTANCE as u32);
        }
    }
}

unsafe fn hash_all() {
    g::hash_HSD_GObj(&GOBJ);
    g::hash_HSD_GObj(&X18G);
    g::hash_HSD_GObj(&CURG);
    g::hash_HSD_GObj(&CAMG);
    g::hash_HSD_GObj(&FOGG);
    for i in 0..3 {
        g::hash_HSD_GObj(&FIG[i]);
    }
    g::hash_Ground(&GP);
    for i in 0..7 {
        g::hash_HSD_JObj(&JT[i]);
    }
    for i in 0..3 {
        g::hash_LbShadow(&SHAD[i]);
    }
}

#[test]
fn grdisplay_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GOBJ, core::mem::size_of::<HSD_GObj>());
        preg(&raw const X18G, core::mem::size_of::<HSD_GObj>());
        preg(&raw const CURG, core::mem::size_of::<HSD_GObj>());
        preg(&raw const CAMG, core::mem::size_of::<HSD_GObj>());
        preg(&raw const FOGG, core::mem::size_of::<HSD_GObj>());
        preg(&raw const FIG, core::mem::size_of_val(&FIG));
        preg(&raw const GP, core::mem::size_of::<Ground>());
        preg(&raw const JT, core::mem::size_of_val(&JT));
        preg(&raw const CAMCO, core::mem::size_of::<HSD_CObj>());
        preg(&raw const OTHCO, core::mem::size_of::<HSD_CObj>());
        preg(&raw const FOGO, 16);
        preg(&raw const SHAD, core::mem::size_of_val(&SHAD));
        preg(&raw const PLINK, core::mem::size_of_val(&PLINK));
        preg(&raw const VM, core::mem::size_of::<Mtx>());
        g::freg_all();
        freg(ocsr as *const () as usize);
        let mut c = Check::new(include_str!("data/mtu_grdisplay.txt"));
        for op in 0..4u32 {
            for k in 0..2500u32 {
                case(op, k);
                setup();
                match op {
                    0 => {
                        let fl = rnd() & 0xF;
                        let rm = fv();
                        let vm = if rnd() & 1 != 0 { VM.as_mut_ptr() } else { null_mut() };
                        m::grDisplay_801C5B90(&raw mut JT[0], vm, fl, rm);
                    }
                    1 => {
                        let code = fv() as i32;
                        m::grDisplay_801C5DB0_surface_impl(&raw mut GOBJ, code);
                    }
                    2 => {
                        let code = fv() as i32;
                        m::grDisplay_801C5DB0(&raw mut GOBJ, code as isize);
                    }
                    _ => {
                        let code = fv() as i32;
                        m::grDisplay_801C5F60(&raw mut CAMG, code as isize);
                    }
                }
                hash_all();
                c.line(op, k, 0);
            }
        }
        assert_eq!(c.done(), 10000);
    }
}
