//! tu::gm_180A against the C TU compiled natively with shipping contraction: the state oracle
//! (gen/turef/tu_gm_180A_ref.c; data in tests/data/tu_gm_180A.txt, regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard529.py gm_180A`). Every callee outside the TU is a
//! tracing stub on both sides, so the call trace (order, arguments) and the resulting statics are
//! compared case by case.
#![allow(static_mut_refs, non_snake_case, unused)]
mod common;
use common::*;
use melee_modes::tu::gm_180A as t;
use melee_modes::tu_ext as ext;
use melee_types::*;

unsafe fn mkchain() -> *mut HSD_JObj {
    let root = az_t::<HSD_JObj>();
    let c1 = az_t::<HSD_JObj>();
    let c2 = az_t::<HSD_JObj>();
    let mut prev = c2;
    (*root).child = c1;
    (*c1).child = c2;
    for _ in 0..7 {
        let n = az_t::<HSD_JObj>();
        (*prev).next = n;
        prev = n;
    }
    root
}

include!("stubs/gm_180A.rs");

unsafe fn sym() -> *mut core::ffi::c_void {
    let t = az(16) as *mut *mut DynamicModelDesc;
    *t = az_t::<DynamicModelDesc>();
    (**t).joint = az(64) as *mut HSD_Joint;
    t as *mut core::ffi::c_void
}
fn t_lbArchive_80016DBC(file: *const core::ffi::c_char, rest: &[*mut core::ffi::c_void]) -> *mut HSD_Archive {
    unsafe {
        out("lbArchive_80016DBC");
        trs(file);
        for pair in rest.chunks(2) {
            trp(pair[0] as *const u8);
            trs(pair[1] as *const core::ffi::c_char);
            *(pair[0] as *mut *mut core::ffi::c_void) = sym();
        }
        out("\n");
        az_t::<HSD_Archive>()
    }
}

unsafe fn state() {
    let s = &t::lbl_80472E48;
    out(&format!("e48 {} {} {} {}\n", s.b76(), s.b54(), s.b32(), s.b10()));
    dump("e48t", (s as *const _ as *const u8).add(1), core::mem::size_of_val(s) - 1);
    dump("ec8", t::lbl_80472EC8.as_ptr() as *const u8, 16);
    out(&format!("g {} {}\n", t::lbl_804D65D4, t::lbl_804D65D8));
    dump("cc", &t::lbl_804D65CC as *const _ as *const u8, 8);
    dump("d0", &t::lbl_804D65D0 as *const _ as *const u8, 8);
    dump("c8", &t::lbl_804D65C8 as *const _ as *const u8, 8);
    dump_arena();
    state_end();
}
unsafe fn seed_e48(smallx: bool) {
    let s = &mut t::lbl_80472E48;
    rfill_t(s);
    s.set_b76((rnd() & 3) as u8);
    s.set_b54((rnd() & 3) as u8);
    s.set_b32((rnd() & 3) as u8);
    s.set_b10((rnd() & 3) as u8);
    s.unk_4 = (rnd() % 300) as i32 - 20;
    s.x10 = (rnd() % 4) as u8;
    s.xC = rnd() % (if smallx { 0x120 } else { 0x40000000 });
    for i in 0..0x1B {
        s.x14[i] = (rnd() % 2500) as i32 - 100;
    }
    for i in 0..4 {
        t::lbl_80472EC8[i] = (rnd() % 2500) as i32 - 100;
    }
}

#[test]
fn matches_c() {
    unsafe {
        reset_all();
        gen_stubs::install();
        ext::HOOK_lbArchive_80016DBC = Some(t_lbArchive_80016DBC);
        ext::HSD_GObj_JObjKind = 0x2A;
        size_t::<lbl_80472E48_t_>("struct lbl_80472E48_t");
        size_t::<HSD_GObj>("HSD_GObj");
        size_t::<HSD_JObj>("HSD_JObj");
        size_t::<DynamicModelDesc>("DynamicModelDesc");
        size_t::<HSD_Archive>("HSD_Archive");
        size_t::<HSD_PadStatus>("HSD_PadStatus");
        reg("e48", &t::lbl_80472E48, core::mem::size_of_val(&t::lbl_80472E48));
        reg("ec8", t::lbl_80472EC8.as_ptr(), 16);
        reg("pad", ext::HSD_PadCopyStatus.as_ptr(), core::mem::size_of_val(&ext::HSD_PadCopyStatus));
        fnreg("fn_80180C14", t::fn_80180C14 as usize);
        fnreg("fn_80180C60", t::fn_80180C60 as usize);
        fnreg("fn_80181598", t::fn_80181598_cb as usize);
        fnreg("HSD_GObj_JObjCallback", ext::HSD_GObj_JObjCallback as usize);

        for c in 0..8usize {
            case(c);
            rseed(100 + c as u32);
            arena_reset();
            seed_e48(false);
            out(&format!("r {}\n", t::gm_80180AE4()));
            out("r");
            let p = t::gm_80180AF4();
            trp(p as *const u8);
            out("\n");
            let (a, b) = (rnd() as i32, rnd() as i32);
            t::gm_80181A00(a, b);
            state();
        }
        for c in 0..8usize {
            case(100 + c);
            rseed(200 + c as u32);
            arena_reset();
            seed_e48(false);
            t::gm_80180B18();
            state();
            t::gm_80180BA0();
            state();
        }
        for c in 0..12usize {
            case(200 + c);
            rseed(300 + c as u32);
            arena_reset();
            seed_e48(false);
            let g = az_t::<HSD_GObj>();
            (*g).hsd_obj = mkchain() as *mut _;
            t::fn_80180C14(g);
            state();
        }
        for c in 0..600usize {
            case(300 + c);
            rseed(1000 + c as u32);
            arena_reset();
            seed_e48(false);
            let s = ORS;
            let gf = rf();
            ORS = s;
            let mut d = (0.1f32 * gf) as i32;
            if d < 0 {
                d = 0;
            }
            if rnd() % 3 == 0 {
                t::lbl_80472EC8[1] = d;
            }
            if rnd() % 3 == 0 {
                t::lbl_80472EC8[3] = (rnd() % 0x80) as i32;
            }
            if rnd() % 4 == 0 {
                t::lbl_80472E48.set_b76(1);
            }
            t::lbl_804D65D4 = (rnd() % 2) as i32;
            let g = az_t::<HSD_GObj>();
            (*g).hsd_obj = mkchain() as *mut _;
            t::fn_80180C60(g);
            state();
        }
        for c in 0..600usize {
            case(4000 + c);
            rseed(9000 + c as u32);
            arena_reset();
            seed_e48(true);
            for i in 0..4 {
                rfill_t(&mut ext::HSD_PadCopyStatus[i]);
                ext::HSD_PadCopyStatus[i].trigger = if rnd() % 2 != 0 { 0x100 } else { 0 };
            }
            t::lbl_804D65D8 = (rnd() % 0x120) as i32;
            if rnd() % 4 == 0 {
                t::lbl_80472E48.set_b10(1);
            }
            if rnd() % 4 == 0 {
                t::lbl_80472E48.set_b10(0);
            }
            if rnd() % 3 == 0 {
                t::lbl_80472E48.set_b32(1);
            }
            t::fn_80181598();
            state();
        }
        for c in 0..6usize {
            case(8000 + c);
            rseed(500 + c as u32);
            arena_reset();
            seed_e48(false);
            t::lbl_804D65CC = sym() as *mut *mut DynamicModelDesc;
            t::lbl_804D65D0 = sym() as *mut *mut DynamicModelDesc;
            t::fn_80181708();
            state();
        }
        for c in 0..4usize {
            case(8100 + c);
            rseed(600 + c as u32);
            arena_reset();
            seed_e48(false);
            t::gm_80181998();
            state();
        }
    }
    check("tu_gm_180A");
}

#[allow(non_camel_case_types)]
type lbl_80472E48_t_ = t::lbl_80472E48_t;
