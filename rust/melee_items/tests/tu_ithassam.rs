//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/ithassam.c.
//! src/tu/ithassam.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py ithassam`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::ithassam as m;
use mtu_harness::*;
#[path = "mtu/ithassam_gen.rs"]
mod g;

pub static mut FG: [HSD_GObj; 3] = unsafe { core::mem::zeroed() };
pub static mut FT: [Fighter; 3] = unsafe { core::mem::zeroed() };
pub static mut PLH: [*mut HSD_GObj; 32] = [null_mut(); 32];




unsafe fn set_uv(op: u32) {
    g::UV_HSD_JObj_u = 0; g::UV_ECBSource_anon_4 = 0; g::UV_CommandInfo_anon_8 = 0; g::UV_ColorOverlay_x28_colanim = 0; g::UV_HitCapsule_anon_134 = 0; g::UV_Item_xD90 = 0; g::UV_Item_anon_daa = 0; g::UV_Item_anon_477_xDAA_flag = 0; g::UV_Item_anon_dbc = 0; g::UV_Item_xDCE_flag = 0; g::UV_Item_xDCF_flag = 0; g::UV_Item_xDD0_flag = 0; g::UV_Item_xDD1_flag = 0; g::UV_Item_xDD2_flag = 0; g::UV_Item_xDD3_flag = 0; g::UV_Item_xDD4_itemVar = 0;
    match op {
        0 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        1 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        2 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        3 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        4 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        5 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        6 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        7 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        8 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        9 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        10 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        11 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        12 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        13 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        14 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        15 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        16 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        17 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        18 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        19 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        20 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        21 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        22 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 33; }
        _ => {}
    }
}

unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;
    for n in 0..6usize { g::P_HSD_JObj[n].flags &= !(1u32 << 17); }
    for n in 0..8usize { g::P_DynamicBoneTable[0].bones[n] = &raw mut g::P_HSD_JObj[(rnd() % 6) as usize]; }
    for n in 0..3usize { FG[n].next = if n < 2 { &raw mut FG[n + 1] } else { null_mut() }; FG[n].user_data = &raw mut FT[n] as *mut c_void; FT[n].set_is_sleeping((rnd() & 1) as _); FT[n].team = (rnd() % 3) as _; }
    PLH[HSD_GOBJ_PLINK_FIGHTER as usize] = &raw mut FG[0]; m::ext::HSD_GObjPLinkHead = &raw mut PLH as *mut *mut HSD_GObj;
}

#[test]
fn ithassam_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_ithassam.txt"));
        let nc: [u32; 23] = [400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400];

        for op in 0..23u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                set_uv(op);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHassam_802CDBE0(a0); }
                    1 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHassam_802CDC5C(a0); }
                    2 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHassam_802CDC60(a0, a1); }
                    3 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHassam_802CDC80(a0); }
                    4 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHassam_802CDCB4(a0); }
                    5 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHassam_UnkMotion0_Anim(a0) as u32; }
                    6 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHassam_UnkMotion0_Phys(a0); }
                    7 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHassam_UnkMotion0_Coll(a0) as u32; }
                    8 => {  let a0: *mut Vec = g::pick_Vec3(0) as *mut Vec; let a1: *mut HSD_GObj = (if rnd() & 3 == 0 { null_mut() } else { &raw mut FG[(rnd() % 3) as usize] }) as *mut HSD_GObj; ret = pi(m::itHassam_802CDE1C(a0, a1)); }
                    9 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHassam_802CDF28(a0); }
                    10 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHassam_802CE008(a0); }
                    11 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHassam_UnkMotion1_Anim(a0) as u32; }
                    12 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHassam_UnkMotion1_Phys(a0); }
                    13 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHassam_UnkMotion1_Coll(a0) as u32; }
                    14 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: i32 = (fv()) as i32; let a2: Item_StateChangeFlags = (fv()) as Item_StateChangeFlags; m::itHassam_802CE400_sub(a0, a1, a2); }
                    15 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHassam_802CE400(a0); }
                    16 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHassam_UnkMotion2_Anim(a0) as u32; }
                    17 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHassam_UnkMotion2_Phys(a0); }
                    18 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHassam_UnkMotion2_Coll(a0) as u32; }
                    19 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802CE640(a0); }
                    20 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHassam_UnkMotion3_Anim(a0) as u32; }
                    21 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHassam_UnkMotion3_Phys(a0); }
                    22 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHassam_UnkMotion3_Coll(a0) as u32; }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 9200);
    }
}
