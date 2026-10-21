//! Night-3 round-4 gr forwarders (gr::forward) against the C (cref/gen_forward_r4_ref.sh ->
//! tests/data/forward_r4_ref.txt): every one of the 220, for six map_ids, compared by the trace
//! of callee calls and their arguments (recorders on both sides; G = the gobj, J = its hsd_obj).
use melee_types::{Ground, HSD_GObj, HSD_JObj};
use melee_world::gr::forward::{call_by_name, Callees, CALLEES, COUNT};
use std::cell::RefCell;
use std::collections::BTreeSet;

const DATA: &str = include_str!("data/forward_r4_ref.txt");

thread_local! {
    static TRACE: RefCell<String> = RefCell::new(String::new());
    static PTRS: RefCell<(usize, usize)> = RefCell::new((0, 0));
}
fn sym(p: usize) -> &'static str {
    let (g, j) = PTRS.with(|x| *x.borrow());
    if p == g { "G" } else if p == j { "J" } else if p == 0 { "0" } else { "?" }
}
fn rec(s: String) { TRACE.with(|t| t.borrow_mut().push_str(&s)); }
unsafe fn zako(d: *mut core::ffi::c_void) { rec(format!(" zako({})", sym(d as usize))) }
unsafe fn umc(g: *mut HSD_GObj) { rec(format!(" umc({})", sym(g as usize))) }
unsafe fn imc(j: *mut HSD_JObj, m: i32) { rec(format!(" imc({},{m})", sym(j as usize))) }
unsafe fn anim(g: *mut HSD_GObj, m: i32, b: i32) { rec(format!(" anim({},{m},{b})", sym(g as usize))) }
unsafe fn lb() { rec(" lb()".to_string()) }

#[test]
fn forward_r4_match_c() {
    *CALLEES.write().unwrap() = Callees {
        grzakogenerator_801cae04: zako, ground_update_map_coll: umc,
        ground_init_map_coll: imc, granime_801c8138: anim, lb_800115f4: lb,
    };
    let mut ground: Ground = unsafe { core::mem::zeroed() };
    let mut jobj = [0u8; 16];
    let mut gobj: HSD_GObj = unsafe { core::mem::zeroed() };
    gobj.user_data = &mut ground as *mut Ground as *mut _;
    gobj.hsd_obj = jobj.as_mut_ptr() as *mut _;
    let g = &mut gobj as *mut HSD_GObj;
    PTRS.with(|x| *x.borrow_mut() = (g as usize, jobj.as_ptr() as usize));
    let mut seen = BTreeSet::new();
    let mut rows = 0;
    for line in DATA.lines() {
        let (head, want) = line.split_once(':').unwrap();
        let v: Vec<&str> = head.split_whitespace().collect();
        let (tu, name, id) = (v[0], v[1], v[2].parse::<i32>().unwrap());
        unsafe { (*(gobj.user_data as *mut Ground)).map_id = id as u32 };
        TRACE.with(|t| t.borrow_mut().clear());
        assert!(unsafe { call_by_name(tu, name, g) }, "not ported: {tu}::{name}");
        let got = TRACE.with(|t| t.borrow().clone());
        assert_eq!(got, want, "{line}");
        seen.insert((tu, name));
        rows += 1;
    }
    assert_eq!(seen.len(), COUNT);
    assert_eq!(rows, 6 * COUNT);
}
