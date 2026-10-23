//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/itharisen.c.
//! src/tu/itharisen.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py itharisen`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itharisen as m;
use mtu_harness::*;
#[path = "mtu/itharisen_gen.rs"]
mod g;





unsafe fn set_uv(op: u32) {
    g::UV_HSD_JObj_u = 0; g::UV_ECBSource_anon_4 = 0; g::UV_CommandInfo_anon_8 = 0; g::UV_ColorOverlay_x28_colanim = 0; g::UV_HitCapsule_anon_134 = 0; g::UV_Item_xD90 = 0; g::UV_Item_anon_daa = 0; g::UV_Item_anon_477_xDAA_flag = 0; g::UV_Item_anon_dbc = 0; g::UV_Item_xDCE_flag = 0; g::UV_Item_xDCF_flag = 0; g::UV_Item_xDD0_flag = 0; g::UV_Item_xDD1_flag = 0; g::UV_Item_xDD2_flag = 0; g::UV_Item_xDD3_flag = 0; g::UV_Item_xDD4_itemVar = 0;
    match op {
        0 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        1 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        2 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        3 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        4 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        5 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        6 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        7 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        8 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        9 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        10 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        11 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        12 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        13 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        14 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        15 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        16 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        17 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        18 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        19 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        20 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        21 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        22 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        23 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        24 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        25 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        26 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        27 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        28 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        29 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        30 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        31 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        32 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; }
        _ => {}
    }
}

unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;
    for n in 0..6usize { g::P_HSD_JObj[n].flags &= !(1u32 << 17); }
}

#[test]
fn itharisen_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_itharisen.txt"));
        let nc: [u32; 33] = [400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400];

        for op in 0..33u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                set_uv(op);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802927E8(a0); }
                    1 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_8029282C(a0); }
                    2 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHarisen_Logic24_Spawned(a0); }
                    3 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_8029287C(a0); }
                    4 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHarisen_UnkMotion0_Anim(a0) as u32; }
                    5 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHarisen_UnkMotion0_Phys(a0); }
                    6 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHarisen_UnkMotion0_Coll(a0) as u32; }
                    7 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_8029290C(a0); }
                    8 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHarisen_UnkMotion8_Anim(a0) as u32; }
                    9 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHarisen_UnkMotion1_Phys(a0); }
                    10 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHarisen_UnkMotion1_Coll(a0) as u32; }
                    11 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: f32 = (rg()) as f32; m::it_80292998(a0, a1); }
                    12 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: f32 = (rg()) as f32; m::it_802929C8(a0, a1); }
                    13 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: f32 = (rg()) as f32; m::it_802929F8(a0, a1); }
                    14 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_80292A28(a0); }
                    15 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHarisen_Logic24_PickedUp(a0); }
                    16 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHarisen_UnkMotion6_Anim(a0) as u32; }
                    17 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHarisen_UnkMotion6_Phys(a0); }
                    18 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHarisen_Logic24_Dropped(a0); }
                    19 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHarisen_UnkMotion8_Coll(a0) as u32; }
                    20 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHarisen_Logic24_Thrown(a0); }
                    21 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHarisen_UnkMotion8_Phys(a0); }
                    22 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHarisen_UnkMotion7_Coll(a0) as u32; }
                    23 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHarisen_Logic24_DmgDealt(a0) as u32; }
                    24 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHarisen_Logic24_EnteredAir(a0); }
                    25 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHarisen_UnkMotion9_Anim(a0) as u32; }
                    26 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHarisen_UnkMotion9_Phys(a0); }
                    27 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHarisen_UnkMotion9_Coll(a0) as u32; }
                    28 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHarisen_Logic24_Clanked(a0) as u32; }
                    29 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHarisen_Logic24_Reflected(a0) as u32; }
                    30 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHarisen_Logic24_HitShield(a0) as u32; }
                    31 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHarisen_Logic24_ShieldBounced(a0) as u32; }
                    32 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHarisen_Logic24_EvtUnk(a0, a1); }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 13200);
    }
}
