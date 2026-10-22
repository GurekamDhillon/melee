//! Rust mirror of gen/turef/tu_gr_common.h (the helpers of the gr TU state oracles).
#![allow(dead_code, static_mut_refs, non_snake_case)]
use super::*;
use core::ffi::c_char;
use melee_types::*;
use melee_world::gr::forward;
use melee_world::tu_ext as ext;

pub unsafe fn mkjobj() -> *mut HSD_JObj {
    let j = az_t::<HSD_JObj>();
    (*j).flags = rnd();
    (*j).translate.x = rf();
    (*j).translate.y = rf();
    (*j).translate.z = rf();
    (*j).scale.x = rf();
    (*j).scale.y = rf();
    (*j).scale.z = rf();
    j
}
/// Every Ground made since `gr_reset()` (see tu_gr_common.h: the bitfield bytes of Ground are
/// hashed only through `gnd_canon_begin/end`).
pub static mut GPS: std::vec::Vec<*mut Ground> = std::vec::Vec::new();
pub unsafe fn gr_reset() {
    arena_reset();
    GPS.clear();
}
pub unsafe fn mkground() -> *mut Ground {
    let g = az_t::<Ground>();
    (*g).map_id = rnd() % 60;
    GPS.push(g);
    g
}
static mut GSV: std::vec::Vec<[u8; 2]> = std::vec::Vec::new();
pub unsafe fn gnd_canon_begin() {
    GSV.clear();
    for (i, &g) in GPS.iter().enumerate() {
        let p10 = (&raw mut (*g).x10_flags) as *mut u8;
        let p11 = (&raw mut (*g).x11_flags) as *mut u8;
        let f = &(*g).x10_flags;
        let h = &(*g).x11_flags;
        out(&format!(
            "g{} x10 {}{}{}{}{}{}{}{} x11 {} {} {} {} {} {}\n",
            i, f.b0(), f.b1(), f.b2(), f.b3(), f.b4(), f.b5(), f.b6(), f.b7(), h.b012(), h.b3(), h.b4(), h.b5(), h.b6(), h.b7()
        ));
        GSV.push([*p10, *p11]);
        *p10 = 0;
        *p11 = 0;
    }
}
pub unsafe fn gnd_canon_end() {
    for (i, &g) in GPS.iter().enumerate() {
        *((&raw mut (*g).x10_flags) as *mut u8) = GSV[i][0];
        *((&raw mut (*g).x11_flags) as *mut u8) = GSV[i][1];
    }
}
pub unsafe fn mkgobj() -> *mut HSD_GObj {
    let g = az_t::<HSD_GObj>();
    (*g).hsd_obj = mkjobj() as *mut _;
    (*g).user_data = mkground() as *mut _;
    g
}

fn trfile(s: *const c_char) {
    if s.is_null() {
        return out(" null");
    }
    let b = unsafe { core::ffi::CStr::from_ptr(s) }.to_bytes();
    let base = match b.iter().rposition(|&c| c == 47 || c == 92) {
        Some(i) => &b[i + 1..],
        None => b,
    };
    out(&format!(" \"{}\"", String::from_utf8_lossy(base)));
}
fn t_OSReport(fmt: *const c_char, args: &[ext::OsArg]) {
    unsafe {
        out("OSReport");
        trs(fmt);
        for a in args {
            match *a {
                ext::OsArg::S(s) => trfile(s),
                ext::OsArg::I(i) => tri(i as i32 as i64),
            }
        }
        out("\n");
    }
}
fn t_grAnime_801C752C(j: *mut HSD_JObj, a: i32, b: i32, f: *mut core::ffi::c_void, n: u32, flags: u32) {
    out("grAnime_801C752C");
    trp(j as *const u8);
    tri(a as i64);
    tri(b as i64);
    trp(f as *const u8);
    tru(n as u64);
    tru(flags as u64);
    out("\n");
}
/// Install the hand-written variadic stubs and route the `gr::forward` forwarders (which
/// call through `forward::CALLEES`) to the same hooks the TU ports use.
pub unsafe fn install_custom() {
    ext::HOOK_OSReport = Some(t_OSReport);
    ext::HOOK_grAnime_801C752C = Some(t_grAnime_801C752C);
    let mut c = forward::CALLEES.write().unwrap();
    c.grzakogenerator_801cae04 = |p| unsafe {
        ext::grZakoGenerator_801CAE04(p as *mut _);
    };
    c.ground_update_map_coll = |g| unsafe {
        ext::Ground_UpdateMapColl(g);
    };
    c.ground_init_map_coll = |j, m| unsafe {
        ext::Ground_InitMapColl(j, m);
    };
    c.granime_801c8138 = |g, m, b| unsafe { ext::grAnime_801C8138(g, m, b) };
    c.lb_800115f4 = || unsafe { ext::lb_800115F4() };
}
