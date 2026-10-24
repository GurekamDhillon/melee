//! tu::lbgx vs the C (state oracle): gen/turef/tu_lbgx_ref.c is the game's lbgx.c compiled natively with
//! every GX call stubbed and logged; expected text in tests/data/tu_lbgx.txt.
#![allow(static_mut_refs, non_snake_case)]
#[macro_use]
#[path = "lbtest/mod.rs"]
mod common_lb;
use common_lb::*;
use melee_modes::tu::lbgx as t;
use melee_types::*;

static mut CO: HSD_CObj = unsafe { core::mem::zeroed() };

fn pid(p: *const core::ffi::c_void) -> &'static str {
    unsafe {
        if p.is_null() {
            "null"
        } else if p == CO.view_mtx.as_ptr() as *const core::ffi::c_void {
            "view"
        } else {
            "?"
        }
    }
}

mod gen_stubs {
    use super::*;
    include!("lbtest/lbgx_stubs.rs");
}

#[test]
fn lbgx() {
    unsafe {
        gen_stubs::install_generated();
        t::ext::H_HSD_CObjGetCurrent = Some(|| {
            lg("cobj();");
            core::ptr::addr_of_mut!(CO)
        });
        for i in 0..120 {
            let sel = if rnd() % 4 == 0 { rnd() % 5 } else { 2 };
            let f1 = rf();
            let v4 = Vec4 { x: rf(), y: rf(), z: rf(), w: rf() };
            let v3 = Vec3 { x: rf(), y: rf(), z: rf() };
            let cl = U8Vec4 { x: rnd() as u8, y: rnd() as u8, z: rnd() as u8, w: rnd() as u8 };
            log_take();
            let k = t::lbGx_8001E2F8(&v4, &v3, &cl, sel, f1);
            outf!("{} sel={} f1={:08x} ret={} log={}\n", i, sel, f1.to_bits(), k as i32, log_take());
        }
    }
    check("tu_lbgx");
}
