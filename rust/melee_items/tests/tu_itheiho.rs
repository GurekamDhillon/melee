//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/itheiho.c.
//! src/tu/itheiho.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py itheiho`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itheiho as m;
use mtu_harness::*;
#[path = "mtu/itheiho_gen.rs"]
mod g;

pub static mut HATTR: [u64; 8] = [0; 8];
pub static mut HINT: i32 = 0;




unsafe fn set_uv(op: u32) {
    g::UV_ECBSource_anon_4 = 0; g::UV_CommandInfo_anon_8 = 0; g::UV_ColorOverlay_x28_colanim = 0; g::UV_HitCapsule_anon_134 = 0; g::UV_Item_xD90 = 0; g::UV_Item_anon_daa = 0; g::UV_Item_anon_477_xDAA_flag = 0; g::UV_Item_anon_dbc = 0; g::UV_Item_xDCE_flag = 0; g::UV_Item_xDCF_flag = 0; g::UV_Item_xDD0_flag = 0; g::UV_Item_xDD1_flag = 0; g::UV_Item_xDD2_flag = 0; g::UV_Item_xDD3_flag = 0; g::UV_Item_xDD4_itemVar = 0; g::UV_itHeiho_ItemVars_x8 = 0;
    match op {
        0 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        1 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        2 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        3 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        4 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        5 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        6 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        7 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        8 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        9 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        10 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        11 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        12 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        13 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        14 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        15 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        16 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        17 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        18 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        19 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        20 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        21 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        22 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        23 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        24 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        25 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        26 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        27 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        28 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 35; }
        _ => {}
    }
}

unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;
    for n in 0..6usize { g::P_HSD_JObj[n].flags &= !(1u32 << 17); }
    for n in 0..8usize { g::P_DynamicBoneTable[0].bones[n] = &raw mut g::P_HSD_JObj[(rnd() % 6) as usize]; }
    for n in 2..16usize { *(&raw mut HATTR as *mut u32).add(n) = rg().to_bits(); }
    HINT = fv() as i32; *(&raw mut HATTR as *mut *mut i32) = &raw mut HINT;
}

#[test]
fn itheiho_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_itheiho.txt"));
        let nc: [u32; 29] = [400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400];

        for op in 0..29u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                set_uv(op);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {  let a0: i32 = (fv()) as i32; let a1: *mut Vec = g::pick_Vec3(0) as *mut Vec; let a2: i32 = (fv()) as i32; let a3: i32 = (fv()) as i32; m::it_802D8618(a0, a1, a2, a3); }
                    1 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802D8688(a0); }
                    2 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802D8894(a0); }
                    3 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeiho_UnkMotion0_Anim(a0) as u32; }
                    4 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeiho_UnkMotion0_Phys(a0); }
                    5 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeiho_UnkMotion0_Coll(a0) as u32; }
                    6 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802D8918(a0); }
                    7 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: i32 = (rnd() % 6) as i32; m::itHeiho_UnkMotion1_Anim_inline(a0, a1); }
                    8 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeiho_UnkMotion1_Anim(a0) as u32; }
                    9 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeiho_UnkMotion1_Phys(a0); }
                    10 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeiho_UnkMotion1_Coll(a0) as u32; }
                    11 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeiho_UnkMotion2_Anim(a0) as u32; }
                    12 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802D8EC8_inline(a0); }
                    13 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeiho_UnkMotion2_Phys(a0); }
                    14 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeiho_UnkMotion2_Coll(a0) as u32; }
                    15 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeiho_UnkMotion3_Anim(a0) as u32; }
                    16 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeiho_UnkMotion3_Phys(a0); }
                    17 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeiho_UnkMotion3_Coll(a0) as u32; }
                    18 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::it_802D8EC8(a0) as u32; }
                    19 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802D9168(a0); }
                    20 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeiho_UnkMotion4_Anim(a0) as u32; }
                    21 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeiho_UnkMotion4_Phys(a0); }
                    22 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeiho_UnkMotion4_Coll(a0) as u32; }
                    23 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802D96B0(a0); }
                    24 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802D9714_inline(a0); }
                    25 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802D9714(a0); }
                    26 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802D98AC(a0); }
                    27 => {  let a0: *mut HSD_JObj = g::pick_HSD_JObj(0) as *mut HSD_JObj; let a1: *mut Item = (&raw mut g::P_Item[(rnd() % 2) as usize]) as *mut Item; m::it_802D98C4(a0, a1); }
                    28 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802D9A0C(a0, a1); }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 11600);
    }
}
