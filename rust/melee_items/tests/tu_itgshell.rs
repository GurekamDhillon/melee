//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/itgshell.c.
//! src/tu/itgshell.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py itgshell`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itgshell as m;
use mtu_harness::*;
#[path = "mtu/itgshell_gen.rs"]
mod g;





unsafe fn set_uv(op: u32) {
    g::UV_HSD_JObj_u = 0; g::UV_ECBSource_anon_4 = 0; g::UV_CommandInfo_anon_8 = 0; g::UV_ColorOverlay_x28_colanim = 0; g::UV_HitCapsule_anon_134 = 0; g::UV_Item_xD90 = 0; g::UV_Item_anon_daa = 0; g::UV_Item_anon_477_xDAA_flag = 0; g::UV_Item_anon_dbc = 0; g::UV_Item_xDCE_flag = 0; g::UV_Item_xDCF_flag = 0; g::UV_Item_xDD0_flag = 0; g::UV_Item_xDD1_flag = 0; g::UV_Item_xDD2_flag = 0; g::UV_Item_xDD3_flag = 0; g::UV_Item_xDD4_itemVar = 0;
    match op {
        0 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        1 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        2 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        3 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        4 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        5 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        6 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        7 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        8 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        9 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        10 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        11 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        12 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        13 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        14 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        15 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        16 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        17 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        18 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        19 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        20 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        21 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        22 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        23 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        24 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        25 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        26 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        27 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        28 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        29 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        30 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        31 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        32 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        33 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        34 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        35 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        36 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        37 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        38 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        39 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        40 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        41 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        42 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        43 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        44 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        45 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 30; }
        _ => {}
    }
}

unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;
    for n in 0..6usize { g::P_HSD_JObj[n].flags &= !(1u32 << 17); }
}

#[test]
fn itgshell_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_itgshell.txt"));
        let nc: [u32; 46] = [400, 400, 400, 3000, 3000, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 3000, 400, 400, 400, 400, 400, 400, 400];

        for op in 0..46u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                set_uv(op);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_8028B8D8(a0); }
                    1 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_8028B988(a0); }
                    2 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_8028BA2C(a0); }
                    3 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_8028BAD8(a0); }
                    4 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_8028BC2C(a0); }
                    5 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGShell_Logic14_Spawned(a0); }
                    6 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_8028BE54(a0); }
                    7 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion0_Anim(a0) as u32; }
                    8 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGshell_UnkMotion0_Phys(a0); }
                    9 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion0_Coll(a0) as u32; }
                    10 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_8028C018(a0); }
                    11 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion1_Anim(a0) as u32; }
                    12 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGshell_UnkMotion1_Phys(a0); }
                    13 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion1_Coll(a0) as u32; }
                    14 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGShell_Logic14_PickedUp(a0); }
                    15 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion2_Anim(a0) as u32; }
                    16 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGshell_UnkMotion2_Phys(a0); }
                    17 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGShell_Logic14_Thrown(a0); }
                    18 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion3_Anim(a0) as u32; }
                    19 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGshell_UnkMotion3_Phys(a0); }
                    20 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion3_Coll(a0) as u32; }
                    21 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGShell_Logic14_Dropped(a0); }
                    22 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion4_Anim(a0) as u32; }
                    23 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGshell_UnkMotion4_Phys(a0); }
                    24 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion4_Coll(a0) as u32; }
                    25 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_8028C3A8(a0); }
                    26 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion6_Anim(a0) as u32; }
                    27 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGshell_UnkMotion6_Phys(a0); }
                    28 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion6_Coll(a0) as u32; }
                    29 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_8028C898(a0); }
                    30 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion8_Anim(a0) as u32; }
                    31 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGshell_UnkMotion8_Phys(a0); }
                    32 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion8_Coll(a0) as u32; }
                    33 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGShell_Logic14_EnteredAir(a0); }
                    34 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion9_Anim(a0) as u32; }
                    35 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGshell_UnkMotion9_Phys(a0); }
                    36 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGshell_UnkMotion9_Coll(a0) as u32; }
                    37 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGShell_Logic14_DmgDealt(a0) as u32; }
                    38 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGShell_Logic14_DmgReceived(a0) as u32; }
                    39 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGShell_Logic14_Reflected(a0) as u32; }
                    40 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::shellHit(a0); }
                    41 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGShell_Logic14_Clanked(a0) as u32; }
                    42 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGShell_Logic14_HitShield(a0) as u32; }
                    43 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGShell_Logic14_ShieldBounced(a0) as u32; }
                    44 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::it_8028CF68(a0) as u32; }
                    45 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGShell_Logic14_EvtUnk(a0, a1); }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 26200);
    }
}
