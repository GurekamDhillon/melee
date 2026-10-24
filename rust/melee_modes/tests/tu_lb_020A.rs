//! src/tu/lb_020A.rs vs the C (gen/cref/tu/lb_020A.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/run_tu.py lb_020A`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::lb_020A as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/lb_020A_gen.rs"]
mod g;

static mut J: [HSD_JObj; 3] = unsafe { core::mem::zeroed() };
static mut IK: IKState = unsafe { core::mem::zeroed() };
static mut V: Vec3 = unsafe { core::mem::zeroed() };
static mut M: [f32; 12] = [0.0; 12];

fn reg<T>(p: *const T) {
    preg(p, core::mem::size_of::<T>());
}

#[test]
fn lb_020A_matches_c() {
    unsafe {
        g::install();
        reg(&raw const J);
        reg(&raw const IK);
        reg(&raw const V);
        reg(&raw const M);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_lb_020A.txt"));
        for op in 0..4u32 {
            for k in 0..500u32 {
                let ret = 0u32;
                case(op, k);
                J = core::mem::zeroed();
                for i in 0..3 {
                    g::fill_HSD_JObj(&mut J[i]);
                    let mut f = 0u32;
                    if rnd() % 3 == 0 {
                        f |= JOBJ_MTX_DIRTY as u32;
                    }
                    if rnd() % 4 == 0 {
                        f |= JOBJ_USER_DEF_MTX as u32;
                    }
                    if rnd() % 2 != 0 {
                        f |= JOBJ_USE_QUATERNION as u32;
                    }
                    if rnd() % 3 == 0 {
                        f |= JOBJ_MTX_INDEP_SRT as u32;
                    }
                    f |= fv() & 0xF;
                    J[i].flags = f;
                }
                if TRACEON {
                    println!("S F {:08X} {:08X} {:08X}", J[0].id as u32, (J[0].id as u64 >> 32) as u32, J[0].flags);
                }
                J[0].parent = &raw mut J[1];
                J[1].parent = &raw mut J[2];
                J[2].parent = core::ptr::null_mut();
                IK = core::mem::zeroed();
                g::fill_IKState(&mut IK);
                IK.jobj0 = &raw mut J[0];
                IK.jobj1 = &raw mut J[1];
                IK.jobj2 = &raw mut J[2];
                g::fill_Vec3(&mut V);
                if rnd() % 5 == 0 {
                    V.x = 0.0;
                    V.y = 0.0;
                    V.z = 0.0;
                }
                for i in 0..12 {
                    M[i] = rg();
                }
                match op {
                    0 => m::fn_80020AEC(&raw mut J[0], (&raw mut M) as *mut Mtx),
                    1 => {
                        let a = rg();
                        let b = rg();
                        let c2 = rg();
                        m::lbBgFlash_80020E38(&raw mut J[0], &raw mut V, a, b, c2)
                    }
                    2 => {
                        let a = rg();
                        m::fn_8002113C(&raw mut J[0], &raw mut V, a)
                    }
                    _ => m::lbBgFlash_80021410(&raw mut IK),
                }
                if TRACEON {
                    for q in 0..core::mem::size_of_val(&J) / 4 {
                        println!("D {:08X}", *((&raw const J as *const u32).add(q)));
                    }
                    for q in 0..core::mem::size_of_val(&IK) / 4 {
                        println!("S {:08X}", *((&raw const IK as *const u32).add(q)));
                    }
                    for q in 0..12 {
                        println!("S {:08X}", M[q].to_bits());
                    }
                }
                g::hash_HSD_JObj(&J[0]);
                g::hash_HSD_JObj(&J[1]);
                g::hash_HSD_JObj(&J[2]);
                g::hash_IKState(&IK);
                g::hash_Vec3(&V);
                for i in 0..12 {
                    hm(M[i].to_bits());
                }
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 4 * 500);
    }
}
