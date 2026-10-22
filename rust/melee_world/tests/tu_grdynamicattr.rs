//! src/tu/grdynamicattr.rs vs the C (gen/mtu/tu/grdynamicattr.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_world/gen/mtu/run_tu.py grdynamicattr`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_types::*;
use melee_world::tu::grdynamicattr as m;
use mtu_harness::*;
#[path = "mtu/grdynamicattr_gen.rs"]
mod g;

static mut V: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static mut LEAK: i32 = -1;

unsafe fn in_free(s: usize) -> bool {
    let mut c = m::grDynamicAttr_804D6964;
    while !c.is_null() {
        if c == &raw mut m::grDynamicAttr_8049EFA0[s] {
            return true;
        }
        c = (*c).next;
    }
    false
}

unsafe fn hash_pool() {
    for i in 0..4 {
        g::hash_grDynamicAttr_UnkStruct(&m::grDynamicAttr_8049EFA0[i]);
    }
    hm(pi(m::grDynamicAttr_804D6960));
    hm(pi(m::grDynamicAttr_804D6964));
}

unsafe fn arrange() {
    let n = (rnd() % 5) as usize;
    let mut order = [0usize, 1, 2, 3];
    let mut i = 3;
    while i > 0 {
        let j = (rnd() % (i as u32 + 1)) as usize;
        order.swap(i, j);
        i -= 1;
    }
    for i in 0..4 {
        g::fill_grDynamicAttr_UnkStruct(&mut m::grDynamicAttr_8049EFA0[i]);
    }
    m::grDynamicAttr_804D6960 = core::ptr::null_mut();
    m::grDynamicAttr_804D6964 = core::ptr::null_mut();
    let mut last: *mut grDynamicAttr_UnkStruct = core::ptr::null_mut();
    for i in 0..n {
        let s = &raw mut m::grDynamicAttr_8049EFA0[order[i]];
        (*s).next = core::ptr::null_mut();
        if !last.is_null() {
            (*last).next = s;
        } else {
            m::grDynamicAttr_804D6960 = s;
        }
        last = s;
        (*s).unk1C = (fv() % 4) as i32 - 1;
    }
    last = core::ptr::null_mut();
    LEAK = -1;
    if n < 4 && (rnd() & 1) != 0 {
        LEAK = order[3] as i32;
    }
    for i in n..4 {
        if order[i] as i32 == LEAK {
            continue;
        }
        let s = &raw mut m::grDynamicAttr_8049EFA0[order[i]];
        (*s).next = core::ptr::null_mut();
        if !last.is_null() {
            (*last).next = s;
        } else {
            m::grDynamicAttr_804D6964 = s;
        }
        last = s;
    }
}

#[test]
fn grdynamicattr_matches_c() {
    unsafe {
        g::install();
        preg(&raw const V, core::mem::size_of::<Vec3>());
        preg(&raw const m::grDynamicAttr_8049EFA0, core::mem::size_of_val(&m::grDynamicAttr_8049EFA0));
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_grdynamicattr.txt"));
        for op in 0..6u32 {
            for k in 0..1500u32 {
                case(op, k);
                let mut ret = 0u32;
                arrange();
                V.x = rw();
                V.y = rw();
                V.z = rw();
                match op {
                    0 => m::grDynamicAttr_801CA0B4(),
                    1 => {
                        let mut id = fv() as i32;
                        if id == -1 {
                            id = 0;
                        }
                        let a0 = fv() as i32;
                        let f = rw();
                        let a3 = fv() as i32;
                        ret = pi(m::grDynamicAttr_801CA0F8(a0, &raw mut V, id as u32, f, a3));
                    }
                    2 => {
                        let mut a = core::ptr::null_mut();
                        let w = fv() % 6;
                        if w < 4 && !in_free(w as usize) {
                            a = &raw mut m::grDynamicAttr_8049EFA0[w as usize];
                        }
                        m::grDynamicAttr_801CA1C0(a);
                    }
                    3 => m::grDynamicAttr_801CA224(),
                    4 => {
                        for j in 0..4 {
                            m::grDynamicAttr_8049EFA0[j].unk18 = rw();
                        }
                        let a1 = fv() as i32;
                        ret = m::grDynamicAttr_801CA284(&raw mut V, a1) as u32;
                    }
                    _ => {
                        m::grDynamicAttr_801CA0B4();
                        for _ in 0..12 {
                            let w = fv() % 4;
                            if w == 0 {
                                let mut id = fv() as i32;
                                if id == -1 {
                                    id = 1;
                                }
                                let a0 = fv() as i32;
                                let f = rw();
                                let a3 = fv() as i32 % 5;
                                hm(pi(m::grDynamicAttr_801CA0F8(a0, &raw mut V, id as u32, f, a3)));
                            } else if w == 1 {
                                let s = (fv() % 4) as usize;
                                if !in_free(s) {
                                    m::grDynamicAttr_801CA1C0(&raw mut m::grDynamicAttr_8049EFA0[s]);
                                }
                            } else if w == 2 {
                                m::grDynamicAttr_801CA224();
                            } else {
                                V.x = rw();
                                V.y = rw();
                                let a1 = fv() as i32;
                                hm(m::grDynamicAttr_801CA284(&raw mut V, a1) as u32);
                            }
                        }
                    }
                }
                hash_pool();
                hm(V.x.to_bits());
                hm(V.y.to_bits());
                hm(V.z.to_bits());
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 9000);
    }
}
